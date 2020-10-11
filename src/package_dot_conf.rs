use crate::cargo_config::CargoAcapMetadata;
use crate::shell_includes;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) struct PackageDotConf {
    #[serde(rename = "APPNAME")]
    pub app_name: String,

    #[serde(rename = "PACKAGENAME")]
    pub display_name: String,

    #[serde(rename = "MENUNAME")]
    pub menu_name: String,

    #[serde(rename = "APPID")]
    pub axis_application_id: String,

    #[serde(rename = "VENDOR")]
    pub vendor: String,

    #[serde(rename = "APPOPTS")]
    pub launch_arguments: Option<String>,

    /// The application's major version, a numerical value.
    #[serde(rename = "APPMAJORVERSION")]
    pub app_major_version: i32,

    /// The application's minor version, a numerical value.
    #[serde(rename = "APPMINORVERSION")]
    pub app_minor_version: i32,

    /// The application's micro version, this is interpreted as a string.
    #[serde(rename = "APPMICROVERSION")]
    pub app_micro_version: Option<String>,

    /// A space-separated list of other files and/or directories to be included in the package.
    /// Files listed here will be copied to the application directory during installation.
    /// OTHERFILES can be used if separate libraries or configuration files are used by the main
    /// program. Leave empty if not required.
    #[serde(rename = "OTHERFILES", serialize_with = "serialize_other_files")]
    pub other_files: Vec<String>,

    /// Specifies if a license page should be displayed in Axis product's the web pages.
    #[serde(rename = "LICENSEPAGE")]
    pub license_page: LicensePage,

    /// Used together with LicensePage::Custom to enable display of the status of a custom license
    /// in the web user interface.
    #[serde(rename = "LICENSE_CHECK_ARGS", skip_serializing_if = "Option::is_none")]
    pub license_check_arguments: Option<String>,

    /// Specifies the file to use for a custom Settings page. The file (settings.html in this
    /// example) must be located in the html/ directory. If SETTINGSPAGEFILE is specified, a link
    /// from from Applications > [application name] > Settings page will direct users to the custom
    /// Settings page. A settings page can for example be used to configure and control the
    /// application.
    #[serde(rename = "SETTINGSPAGEFILE", skip_serializing_if = "Option::is_none")]
    pub settings_page_file: Option<String>,

    /// The text displayed on the link to the custom Settings page defined by SETTINGSPAGEFILE.
    #[serde(rename = "SETTINGSPAGETEXT", skip_serializing_if = "Option::is_none")]
    pub settings_page_text: Option<String>,

    /// Specifies a link to the vendor's homepage. For an example, see Web pages overview on the
    /// Main page.
    ///
    /// This should be an HTML snippet, like `<a href="foo">Foo</a>`.
    #[serde(rename = "VENDORHOMEPAGELINK", skip_serializing_if = "Option::is_none")]
    pub vendor_homepage_link: Option<String>,

    /// A filename containing a list of CGI's that an http-enabled application will use. See example
    /// application ax_http_serve_request.c for more information.
    #[serde(rename = "HTTPCGIPATHS", skip_serializing_if = "Option::is_none")]
    pub http_cgi_paths: Option<String>,

    /// A script that will be executed on the Axis product when the installation is completed. The
    /// script must be a shell script located in the same directory as the package.conf file. The
    /// script will be executed from the application directory in the Axis product.
    #[serde(rename = "POSTINSTALLSCRIPT", skip_serializing_if = "Option::is_none")]
    pub post_install_script: Option<String>,

    /// Specifies the minimum required SDK version that the product running the application must
    /// support. Firmware version 5.60 correspond to REQEMBDEVVERSION="2.0"
    #[serde(rename = "REQEMBDEVVERSION")]
    pub required_embedded_development_version: String,

    /// The UNIX user in which to run the application. `"sdk"` is the recommended value.
    #[serde(rename = "APPUSR")]
    pub unix_user: String,

    /// The UNIX group in which to run the application. `"sdk"` is the recommended value.
    #[serde(rename = "APPGRP")]
    pub unix_group: String,

    /// Defines how the application is started.
    #[serde(rename = "STARTMODE")]
    pub start_mode: StartMode,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum LicensePage {
    Axis,
    Custom,
    None,
}

#[serde(rename_all = "snake_case")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum StartMode {
    Respawn,
    Once,
    Never,
}

impl From<cargo::core::Package> for PackageDotConf {
    fn from(package: cargo::core::Package) -> Self {
        let acap_metadata = package
            .manifest()
            .custom_metadata()
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("acap"))
            .map(|v| {
                CargoAcapMetadata::deserialize(v.clone())
                    .expect("error parsing [package.metadata.acap] table")
            });

        let CargoAcapMetadata {
            app_name,
            display_name,
            menu_name,
            vendor,
            axis_application_id,
            vendor_homepage_url,
            launch_arguments,
            license_check_arguments,
            start_mode,
            targets: _,
        } = acap_metadata.unwrap_or(CargoAcapMetadata::default());

        let app_name = app_name.unwrap_or_else(|| package.name().to_string());
        let display_name = display_name.unwrap_or_else(|| package.name().to_string());
        let menu_name = menu_name.unwrap_or_else(|| display_name.clone());

        let vendor = vendor.unwrap_or_else(|| format!("{} authors", &display_name));

        let license_page = if axis_application_id.is_some() {
            LicensePage::Axis
        } else if license_check_arguments.is_some() {
            LicensePage::Custom
        } else {
            LicensePage::None
        };

        // TODO: HTML escaping
        let vendor_homepage_link =
            vendor_homepage_url.map(|url| format!("<a href=\"{}\">{}</a>", url, &vendor));

        let start_mode = start_mode.unwrap_or(StartMode::Respawn);

        let version = package.version();
        let app_major_version = version
            .major
            .try_into()
            .unwrap_or_else(|_| panic!("version {:?} out of range"));
        let app_minor_version = version
            .minor
            .try_into()
            .unwrap_or_else(|_| panic!("version {:?} out of range"));

        let app_micro_version = {
            let mut s = version.patch.to_string();
            for pre in version.pre.iter() {
                s += "-";
                s += &pre.to_string();
            }
            for build in version.build.iter() {
                s += "+";
                s += &build.to_string();
            }
            Some(s)
        };

        PackageDotConf {
            app_name,
            display_name,
            menu_name,
            axis_application_id: axis_application_id.unwrap_or_default(),
            vendor,
            launch_arguments,
            app_major_version,
            app_minor_version,
            app_micro_version,
            other_files: vec![],
            license_page,
            license_check_arguments,
            settings_page_file: None,
            settings_page_text: None,
            vendor_homepage_link,
            http_cgi_paths: None,
            post_install_script: None,
            required_embedded_development_version: "2.0".to_string(),
            unix_user: "sdk".to_string(),
            unix_group: "sdk".to_string(),
            start_mode,
        }
    }
}

impl fmt::Display for PackageDotConf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&shell_includes::to_string(self))
    }
}

fn serialize_other_files<S>(other_files: &Vec<String>, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(bad) = other_files.iter().find(|f| f.contains(" ")) {
        panic!(
            "unable to serialize other_files=[{:?}] since it contains a space",
            bad
        );
    }

    ser.serialize_str(&other_files.join(" "))
}
