use crate::package_dot_conf::StartMode;
use crate::target::Target;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CargoAcapMetadata {
    /// The machine-friendly name of the package. Used for:
    ///
    /// * Installation path: `/usr/local/packages/<app_name>`
    /// * Executable path: `/usr/local/packages/<app_name>/<app_name>`
    /// * Generated package names: `<app_name>_1_2_3_arch.eap`
    /// * Myriad related files
    pub app_name: Option<String>,

    /// A user-friendly package name. The name will be displayed in the Axis product's web pages.
    pub display_name: Option<String>,

    /// The application name that will be displayed in the web pages' left-hand side menu.
    pub menu_name: Option<String>,

    /// The name of the vendor that created the application.
    pub vendor: Option<String>,

    /// The URL of the vendor's home page, to be linked in the product's web pages.
    pub vendor_homepage_url: Option<String>,

    /// The command line arguments to pass when the application is launched.
    pub launch_arguments: Option<String>,

    /// The command line arguments to pass when the application is executed to perform a custom
    /// license check, if using custom licensing.
    pub license_check_arguments: Option<String>,

    /// The Axis-assigned application ID, if using Axis licensing.
    pub axis_application_id: Option<String>,

    /// The start mode to use for this application.
    pub start_mode: Option<StartMode>,

    /// The targets to be built by a bare `cargo acap build` invocation.
    pub targets: Option<Vec<Target>>,
}
