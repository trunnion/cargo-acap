use crate::cli::Invocation;
use crate::target::{Target, SOC};
use clap::Parser;

/// List which targets
#[derive(Parser)]
pub struct Targets {
    /// Which mode to list
    #[clap(arg_enum)]
    mode: Option<Mode>,
}

#[derive(Debug, Clone, Parser, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum Mode {
    /// A plain text list of all supported --target= values
    #[clap(name = "plain")]
    Plain,
    /// A table of targets
    #[clap(name = "table")]
    Table,
    /// A table of SOCs and their associated targets
    #[clap(name = "soc_table")]
    SocTable,
}

impl Targets {
    pub fn invoke(self, _invocation: Invocation) {
        match self.mode.unwrap_or(Mode::Plain) {
            Mode::Plain => {
                for target in Target::all() {
                    println!("{}", target.name());
                }
            }
            Mode::Table => {
                print_table(
                    &["`cargo acap` `target`", "Rust `--target`"],
                    Target::all().iter().map(|t| {
                        vec![
                            format!("`{}`", t.name()),
                            format!("`{}`", t.rust_target_triple()),
                        ]
                    }),
                );
            }
            Mode::SocTable => {
                let mut socs: Vec<&SOC> = SOC::all().iter().collect();
                socs.sort_by_key(|soc| (soc.year(), soc.display_name()));

                print_table(
                    &["SOC", "Year", "`cargo acap` `target`", "Rust `--target`"],
                    socs.into_iter().map(|soc| {
                        let target = soc.architecture().ok();
                        vec![
                            soc.display_name().to_string(),
                            format!("{}", soc.year()),
                            target
                                .map(|t| format!("`{}`", t.name()))
                                .unwrap_or_else(|| "(unsupported)".to_string()),
                            target
                                .map(|t| format!("`{}`", t.rust_target_triple()))
                                .unwrap_or_else(|| "(unsupported)".to_string()),
                        ]
                    }),
                );
            }
        }
    }
}

fn print_table<H, HS, D, R, RS>(headers: H, data: D)
where
    H: IntoIterator<Item = HS>,
    HS: AsRef<str>,
    D: IntoIterator<Item = R>,
    R: IntoIterator<Item = RS>,
    RS: AsRef<str>,
{
    let headers: Vec<HS> = headers.into_iter().collect();

    let mut column_lengths: Vec<usize> = headers.iter().map(|c| c.as_ref().len()).collect();

    let data: Vec<Vec<RS>> = data
        .into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(column, cell)| {
                    let len = cell.as_ref().len();
                    let known_len = &mut column_lengths[column];
                    if *known_len < len {
                        *known_len = len;
                    }
                    cell
                })
                .collect()
        })
        .collect();

    fn print_row<R: IntoIterator<Item = T>, T: AsRef<str>>(row: R, column_lengths: &[usize]) {
        let total_length: usize = column_lengths.iter().map(|l| *l + 2).sum();
        let mut output = String::with_capacity(total_length + 2 + 1);
        output.push('|');

        for (len, cell) in column_lengths.iter().zip(row) {
            let cell: &str = cell.as_ref();
            output.push(' ');
            output.push_str(cell);
            for _ in cell.len()..*len {
                output.push(' ');
            }
            output.push_str(" |");
        }
        output.push('\n');
        print!("{}", output);
    }

    print_row(headers, &column_lengths);
    print_row(
        column_lengths.iter().map(|len| {
            let len = *len;
            let mut dashes = String::with_capacity(len);
            for _ in 0..len {
                dashes.push('-');
            }
            dashes
        }),
        &column_lengths,
    );
    for row in data {
        print_row(row, &column_lengths);
    }
}
