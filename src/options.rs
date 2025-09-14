use crate::dfixxer_error::DFixxerError;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UsesSectionStyle {
    CommaAtTheBeginning,
    CommaAtTheEnd,
}

impl Default for UsesSectionStyle {
    fn default() -> Self {
        UsesSectionStyle::CommaAtTheEnd
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SpaceOperation {
    NoChange,
    Before,
    After,
    BeforeAndAfter,
}

impl Default for SpaceOperation {
    fn default() -> Self {
        SpaceOperation::After
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum LineEnding {
    Auto,
    Crlf,
    Lf,
}

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::Auto
    }
}

impl LineEnding {
    /// Convert the LineEnding enum to the actual line ending string
    pub fn to_string(&self) -> String {
        match self {
            LineEnding::Auto => {
                #[cfg(windows)]
                return "\r\n".to_string();
                #[cfg(not(windows))]
                return "\n".to_string();
            }
            LineEnding::Crlf => "\r\n".to_string(),
            LineEnding::Lf => "\n".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TextChangeOptions {
    pub comma: SpaceOperation,
    pub semi_colon: SpaceOperation,
    pub lt: SpaceOperation,            // '<'
    pub eq: SpaceOperation,            // '='
    pub neq: SpaceOperation,           // '<>'
    pub gt: SpaceOperation,            // '>'
    pub lte: SpaceOperation,           // '<='
    pub gte: SpaceOperation,           // '>='
    pub add: SpaceOperation,           // '+'
    pub sub: SpaceOperation,           // '-'
    pub mul: SpaceOperation,           // '*'
    pub fdiv: SpaceOperation,          // '/'
    pub assign: SpaceOperation,        // ':='
    pub assign_add: SpaceOperation,    // '+='
    pub assign_sub: SpaceOperation,    // '-='
    pub assign_mul: SpaceOperation,    // '*='
    pub assign_div: SpaceOperation,    // '/='
    pub colon: SpaceOperation,         // ':'
    pub colon_numeric_exception: bool, // Skip spacing for ':' when numbers before and after
    pub trim_trailing_whitespace: bool,
}

impl Default for TextChangeOptions {
    fn default() -> Self {
        TextChangeOptions {
            comma: SpaceOperation::After,
            semi_colon: SpaceOperation::After,
            lt: SpaceOperation::BeforeAndAfter,         // '<'
            eq: SpaceOperation::BeforeAndAfter,         // '='
            neq: SpaceOperation::BeforeAndAfter,        // '<>'
            gt: SpaceOperation::BeforeAndAfter,         // '>'
            lte: SpaceOperation::BeforeAndAfter,        // '<='
            gte: SpaceOperation::BeforeAndAfter,        // '>='
            add: SpaceOperation::BeforeAndAfter,        // '+'
            sub: SpaceOperation::BeforeAndAfter,        // '-'
            mul: SpaceOperation::BeforeAndAfter,        // '*'
            fdiv: SpaceOperation::BeforeAndAfter,       // '/'
            assign: SpaceOperation::BeforeAndAfter,     // ':='
            assign_add: SpaceOperation::BeforeAndAfter, // '+='
            assign_sub: SpaceOperation::BeforeAndAfter, // '-='
            assign_mul: SpaceOperation::BeforeAndAfter, // '*='
            assign_div: SpaceOperation::BeforeAndAfter, // '/='
            colon: SpaceOperation::After,               // ':'
            colon_numeric_exception: true, // Skip spacing for ':' when numbers before and after
            trim_trailing_whitespace: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TransformationOptions {
    pub enable_uses_section: bool,
    pub enable_unit_program_section: bool,
    pub enable_single_keyword_sections: bool,
    pub enable_procedure_section: bool,
    pub enable_text_transformations: bool,
}

impl Default for TransformationOptions {
    fn default() -> Self {
        TransformationOptions {
            enable_uses_section: true,
            enable_unit_program_section: true,
            enable_single_keyword_sections: true,
            enable_procedure_section: true,
            enable_text_transformations: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    pub indentation: String,
    pub uses_section_style: UsesSectionStyle,
    pub override_sorting_order: Vec<String>,
    pub module_names_to_update: Vec<String>,
    pub line_ending: LineEnding,
    pub transformations: TransformationOptions,
    pub text_changes: TextChangeOptions,
    pub exclude_files: Vec<String>,
    pub custom_config_patterns: Vec<(String, String)>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            indentation: "  ".to_string(),
            uses_section_style: UsesSectionStyle::CommaAtTheEnd,
            override_sorting_order: Vec::new(),
            exclude_files: Vec::new(),
            custom_config_patterns: Vec::new(),
            module_names_to_update: vec![
                "System:Actions".to_string(),
                "System:Analytics.AppAnalytics".to_string(),
                "System:Analytics".to_string(),
                "System:AnsiStrings".to_string(),
                "System:Character".to_string(),
                "System:Classes".to_string(),
                "System:Contnrs".to_string(),
                "System:ConvUtils".to_string(),
                "System:Curl".to_string(),
                "System:DateUtils".to_string(),
                "System:Devices".to_string(),
                "System:Diagnostics".to_string(),
                "System:Generics.Collections".to_string(),
                "System:Generics.Defaults".to_string(),
                "System:Hash".to_string(),
                "System:HelpIntfs".to_string(),
                "System:IOUtils".to_string(),
                "System:ImageList".to_string(),
                "System:IniFiles".to_string(),
                "System:Internal.DebugUtils".to_string(),
                "System:Internal.ICU".to_string(),
                "System:JSON.BSON".to_string(),
                "System:JSON.Builders".to_string(),
                "System:JSON.Converters".to_string(),
                "System:JSON.Readers".to_string(),
                "System:JSON.Serializers".to_string(),
                "System:JSON.Types".to_string(),
                "System:JSON.Utils".to_string(),
                "System:JSON.Writers".to_string(),
                "System:JSON".to_string(),
                "System:JSONConsts".to_string(),
                "System:MaskUtils".to_string(),
                "System:Masks".to_string(),
                "System:Math.Vectors".to_string(),
                "System:Math".to_string(),
                "System:Messaging".to_string(),
                "System:NetEncoding.Sqids".to_string(),
                "System:NetEncoding".to_string(),
                "System:Notification".to_string(),
                "System:ObjAuto".to_string(),
                "System:Odbc".to_string(),
                "System:Permissions".to_string(),
                "System:PushNotification".to_string(),
                "System:RTLConsts".to_string(),
                "System:RegularExpressions".to_string(),
                "System:RegularExpressionsAPI".to_string(),
                "System:RegularExpressionsConsts".to_string(),
                "System:RegularExpressionsCore".to_string(),
                "System:Rtti".to_string(),
                "System:Sensors.Components".to_string(),
                "System:Sensors".to_string(),
                "System:Skia.API".to_string(),
                "System:Skia".to_string(),
                "System:Sqlite".to_string(),
                "System:StartUpCopy".to_string(),
                "System:StdConvs".to_string(),
                "System:StrUtils".to_string(),
                "System:SyncObjs".to_string(),
                "System:SysUtils".to_string(),
                "System:Threading".to_string(),
                "System:TimeSpan".to_string(),
                "System:TypInfo".to_string(),
                "System:UIConsts".to_string(),
                "System:UITypes".to_string(),
                "System:VarCmplx".to_string(),
                "System:VarConv".to_string(),
                "System:Vulkan".to_string(),
                "System:WideStrUtils".to_string(),
                "System:WideStrings".to_string(),
                "System:Win.ComConst".to_string(),
                "System:Win.ComObj".to_string(),
                "System:Win.ComObjWrapper".to_string(),
                "System:Win.ComServ".to_string(),
                "System:Win.Crtl".to_string(),
                "System:Win.Devices".to_string(),
                "System:Win.HighDpi".to_string(),
                "System:Win.IEInterfaces".to_string(),
                "System:Win.InternetExplorer".to_string(),
                "System:Win.Mtsobj".to_string(),
                "System:Win.Notification".to_string(),
                "System:Win.ObjComAuto".to_string(),
                "System:Win.OleControls".to_string(),
                "System:Win.OleServers".to_string(),
                "System:Win.Registry".to_string(),
                "System:Win.ScktComp".to_string(),
                "System:Win.Sensors".to_string(),
                "System:Win.ShareContract".to_string(),
                "System:Win.StdVCL".to_string(),
                "System:Win.Taskbar".to_string(),
                "System:Win.TaskbarCore".to_string(),
                "System:Win.VCLCom".to_string(),
                "System:Win.WinRT".to_string(),
                "System:ZLib".to_string(),
                "System:ZLibConst".to_string(),
                "System:Zip".to_string(),
                "System.Win:ComConst".to_string(),
                "System.Win:ComObj".to_string(),
                "System.Win:ComObjWrapper".to_string(),
                "System.Win:ComServ".to_string(),
                "System.Win:Crtl".to_string(),
                "System.Win:Devices".to_string(),
                "System.Win:HighDpi".to_string(),
                "System.Win:IEInterfaces".to_string(),
                "System.Win:InternetExplorer".to_string(),
                "System.Win:Mtsobj".to_string(),
                "System.Win:Notification".to_string(),
                "System.Win:ObjComAuto".to_string(),
                "System.Win:OleControls".to_string(),
                "System.Win:OleServers".to_string(),
                "System.Win:Registry".to_string(),
                "System.Win:ScktComp".to_string(),
                "System.Win:Sensors".to_string(),
                "System.Win:ShareContract".to_string(),
                "System.Win:StdVCL".to_string(),
                "System.Win:Taskbar".to_string(),
                "System.Win:TaskbarCore".to_string(),
                "System.Win:VCLCom".to_string(),
                "System.Win:WinRT".to_string(),
                "Winapi:ADOInt".to_string(),
                "Winapi:AccCtrl".to_string(),
                "Winapi:AclAPI".to_string(),
                "Winapi:ActiveX".to_string(),
                "Winapi:AspTlb".to_string(),
                "Winapi:Bluetooth".to_string(),
                "Winapi:BluetoothLE".to_string(),
                "Winapi:COMAdmin".to_string(),
                "Winapi:ComSvcs".to_string(),
                "Winapi:CommCtrl".to_string(),
                "Winapi:CommDlg".to_string(),
                "Winapi:Cor".to_string(),
                "Winapi:CorError".to_string(),
                "Winapi:CorHdr".to_string(),
                "Winapi:Cpl".to_string(),
                "Winapi:D2D1".to_string(),
                "Winapi:D3D10".to_string(),
                "Winapi:D3D10_1".to_string(),
                "Winapi:D3D11".to_string(),
                "Winapi:D3D11Shader".to_string(),
                "Winapi:D3D11Shadertracing".to_string(),
                "Winapi:D3D11_1".to_string(),
                "Winapi:D3D11_2".to_string(),
                "Winapi:D3D11_3".to_string(),
                "Winapi:D3D11on12".to_string(),
                "Winapi:D3D11sdklayers".to_string(),
                "Winapi:D3D12".to_string(),
                "Winapi:D3D12Shader".to_string(),
                "Winapi:D3D12sdklayers".to_string(),
                "Winapi:D3DCommon".to_string(),
                "Winapi:D3DCompiler".to_string(),
                "Winapi:D3DX10".to_string(),
                "Winapi:D3DX8".to_string(),
                "Winapi:D3DX9".to_string(),
                "Winapi:DDEml".to_string(),
                "Winapi:DX7toDX8".to_string(),
                "Winapi:DXFile".to_string(),
                "Winapi:DXGI".to_string(),
                "Winapi:DXGI1_2".to_string(),
                "Winapi:DXGI1_3".to_string(),
                "Winapi:DXGI1_4".to_string(),
                "Winapi:DXTypes".to_string(),
                "Winapi:Direct3D.PkgHelper".to_string(),
                "Winapi:Direct3D".to_string(),
                "Winapi:Direct3D8".to_string(),
                "Winapi:Direct3D9".to_string(),
                "Winapi:DirectDraw".to_string(),
                "Winapi:DirectInput".to_string(),
                "Winapi:DirectMusic".to_string(),
                "Winapi:DirectPlay8".to_string(),
                "Winapi:DirectShow9".to_string(),
                "Winapi:DirectSound".to_string(),
                "Winapi:Dlgs".to_string(),
                "Winapi:DwmApi".to_string(),
                "Winapi:DxDiag".to_string(),
                "Winapi:DxgiFormat".to_string(),
                "Winapi:DxgiType".to_string(),
                "Winapi:EdgeUtils".to_string(),
                "Winapi:FlatSB".to_string(),
                "Winapi:Functiondiscovery".to_string(),
                "Winapi:GDIPAPI".to_string(),
                "Winapi:GDIPOBJ".to_string(),
                "Winapi:GDIPUTIL".to_string(),
                "Winapi:ImageHlp".to_string(),
                "Winapi:Imm".to_string(),
                "Winapi:IpExport".to_string(),
                "Winapi:IpHlpApi".to_string(),
                "Winapi:IpRtrMib".to_string(),
                "Winapi:IpTypes".to_string(),
                "Winapi:Isapi".to_string(),
                "Winapi:Isapi2".to_string(),
                "Winapi:KnownFolders".to_string(),
                "Winapi:LZExpand".to_string(),
                "Winapi:Locationapi".to_string(),
                "Winapi:MLang".to_string(),
                "Winapi:MMSystem".to_string(),
                "Winapi:Manipulations".to_string(),
                "Winapi:Mapi".to_string(),
                "Winapi:Messages".to_string(),
                "Winapi:MsCTF.PkgHelper".to_string(),
                "Winapi:MsCTF".to_string(),
                "Winapi:MsInkAut".to_string(),
                "Winapi:MsInkAut15".to_string(),
                "Winapi:Mshtmhst".to_string(),
                "Winapi:Mtx".to_string(),
                "Winapi:MultiMon".to_string(),
                "Winapi:Nb30".to_string(),
                "Winapi:ObjectArray".to_string(),
                "Winapi:Ole2".to_string(),
                "Winapi:OleCtl".to_string(),
                "Winapi:OleDB".to_string(),
                "Winapi:OleDlg".to_string(),
                "Winapi:OpenGL.PkgHelper".to_string(),
                "Winapi:OpenGL".to_string(),
                "Winapi:OpenGLext".to_string(),
                "Winapi:PenInputPanel".to_string(),
                "Winapi:Penwin".to_string(),
                "Winapi:Portabledevicetypes".to_string(),
                "Winapi:PropKey".to_string(),
                "Winapi:PropSys".to_string(),
                "Winapi:PsAPI".to_string(),
                "Winapi:Qos".to_string(),
                "Winapi:RegStr".to_string(),
                "Winapi:RichEdit".to_string(),
                "Winapi:RtsCom".to_string(),
                "Winapi:SHFolder".to_string(),
                "Winapi:Sensors".to_string(),
                "Winapi:Sensorsapi".to_string(),
                "Winapi:ShLwApi".to_string(),
                "Winapi:ShellAPI".to_string(),
                "Winapi:ShellScaling".to_string(),
                "Winapi:ShlObj".to_string(),
                "Winapi:StructuredQuery".to_string(),
                "Winapi:StructuredQueryCondition".to_string(),
                "Winapi:TlHelp32".to_string(),
                "Winapi:TpcShrd".to_string(),
                "Winapi:UrlMon".to_string(),
                "Winapi:UserEnv".to_string(),
                "Winapi:UxTheme".to_string(),
                "Winapi:Vulkan".to_string(),
                "Winapi:WMF9".to_string(),
                "Winapi:WTSApi32".to_string(),
                "Winapi:Wbem".to_string(),
                "Winapi:WebView2".to_string(),
                "Winapi:WinCred".to_string(),
                "Winapi:WinHTTP".to_string(),
                "Winapi:WinInet".to_string(),
                "Winapi:WinSock".to_string(),
                "Winapi:WinSpool".to_string(),
                "Winapi:WinSvc".to_string(),
                "Winapi:Wincodec".to_string(),
                "Winapi:Windows.PkgHelper".to_string(),
                "Winapi:Windows".to_string(),
                "Winapi:Winrt".to_string(),
                "Winapi:WinrtMetadata".to_string(),
                "Winapi:Winsafer".to_string(),
                "Winapi:Winsock2".to_string(),
                "Winapi:msxml".to_string(),
                "Winapi:msxmlIntf".to_string(),
                "Winapi:oleacc".to_string(),
            ],
            line_ending: LineEnding::Auto,
            transformations: TransformationOptions::default(),
            text_changes: TextChangeOptions::default(),
        }
    }
}

/// Check if a file path matches any of the given glob patterns
///
/// Patterns are matched relative to the configuration file's directory.
///
/// # Arguments
/// * `patterns` - A slice of glob patterns to match against
/// * `file_path` - The absolute or relative path to the file to check
/// * `config_path` - The path to the configuration file (for determining base directory)
///
/// # Returns
/// * `Some(pattern)` if the file matches a pattern, `None` otherwise
fn match_file_patterns(patterns: &[String], file_path: &str, config_path: Option<&str>) -> Option<String> {
    if patterns.is_empty() {
        return None;
    }

    let file_path = Path::new(file_path);

    // Get the base directory from the config file path
    let base_dir = if let Some(config) = config_path {
        Path::new(config)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(".")
    };

    // Try to make the file path relative to the base directory
    let relative_path = if file_path.is_absolute() {
        file_path
            .strip_prefix(&base_dir)
            .unwrap_or(file_path)
    } else {
        file_path
    };

    // Convert to string for pattern matching, normalize separators to forward slashes
    let path_str = relative_path
        .to_string_lossy()
        .replace('\\', "/");

    // Check each pattern
    for pattern_str in patterns {
        match Pattern::new(pattern_str) {
            Ok(pattern) => {
                if pattern.matches(&path_str) {
                    log::debug!("File '{}' matched pattern '{}'", path_str, pattern_str);
                    return Some(pattern_str.clone());
                }
            }
            Err(e) => {
                log::warn!("Invalid glob pattern '{}': {}", pattern_str, e);
            }
        }
    }

    None
}

/// Check if a file should be excluded based on exclude_files patterns
///
/// Patterns are matched relative to the configuration file's directory.
///
/// # Arguments
/// * `exclude_patterns` - A slice of glob patterns to match against
/// * `file_path` - The absolute or relative path to the file to check
/// * `config_path` - The path to the configuration file (for determining base directory)
///
/// # Returns
/// * `true` if the file should be excluded, `false` otherwise
pub fn should_exclude_file(exclude_patterns: &[String], file_path: &str, config_path: Option<&str>) -> bool {
    if let Some(pattern) = match_file_patterns(exclude_patterns, file_path, config_path) {
        log::info!("File '{}' excluded by pattern '{}'", file_path, pattern);
        true
    } else {
        false
    }
}

/// Find a custom configuration file for a file based on custom_config_patterns
///
/// Patterns are matched relative to the configuration file's directory.
///
/// # Arguments
/// * `custom_patterns` - A slice of (pattern, config_path) pairs
/// * `file_path` - The absolute or relative path to the file to check
/// * `config_path` - The path to the configuration file (for determining base directory)
///
/// # Returns
/// * `Some(config_path)` if the file matches a pattern, `None` otherwise
pub fn find_custom_config_for_file(custom_patterns: &[(String, String)], file_path: &str, config_path: Option<&str>) -> Option<String> {
    if custom_patterns.is_empty() {
        return None;
    }

    let patterns: Vec<String> = custom_patterns.iter().map(|(pattern, _)| pattern.clone()).collect();

    if let Some(matched_pattern) = match_file_patterns(&patterns, file_path, config_path) {
        // Find the config path for the matched pattern
        for (pattern, custom_config_path) in custom_patterns {
            if pattern == &matched_pattern {
                // Resolve the custom config path relative to the current config's directory if it's relative
                let resolved_path = if Path::new(custom_config_path).is_absolute() {
                    custom_config_path.clone()
                } else if let Some(current_config) = config_path {
                    // Make it relative to the current config file's directory
                    if let Some(config_dir) = Path::new(current_config).parent() {
                        config_dir.join(custom_config_path).to_string_lossy().to_string()
                    } else {
                        custom_config_path.clone()
                    }
                } else {
                    custom_config_path.clone()
                };

                log::info!("File '{}' matched custom config pattern '{}', using config '{}'",
                          file_path, pattern, resolved_path);
                return Some(resolved_path);
            }
        }
    }

    None
}

impl Options {
    /// Load options from a TOML file, using defaults for missing fields
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, DFixxerError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DFixxerError::ConfigError(format!("Failed to read config file: {}", e)))?;
        let options: Options = toml::from_str(&content).map_err(|e| {
            DFixxerError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;

        // If uses_section_style is not set, use default
        // (TOML deserialization will use default if missing, but for robustness)
        // If you want to handle string values, you can add custom logic here.

        Ok(options)
    }

    /// Create a default configuration file
    pub fn create_default_config<P: AsRef<Path>>(path: P) -> Result<(), DFixxerError> {
        let default_options = Self::default();
        default_options.save_to_file(path)?;
        Ok(())
    }

    /// Load options from a TOML file, or return default if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(options) => options,
            Err(_) => Self::default(),
        }
    }

    /// Save options to a TOML file
    fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DFixxerError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| DFixxerError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content).map_err(|e| {
            DFixxerError::ConfigError(format!("Failed to write config file: {}", e))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// Helper to create a unique temp directory for tests
    fn create_unique_temp_dir() -> std::path::PathBuf {
        let mut temp_path = env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_path.push(format!("dfixxer_test_{}", unique));
        fs::create_dir_all(&temp_path).unwrap();
        temp_path
    }
    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_default_options() {
        let options = Options::default();
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.uses_section_style, UsesSectionStyle::CommaAtTheEnd);
        assert_eq!(options.override_sorting_order, Vec::<String>::new());
        assert_eq!(options.exclude_files, Vec::<String>::new());
        assert_eq!(options.custom_config_patterns, Vec::<(String, String)>::new());
        assert!(!options.module_names_to_update.is_empty());
        assert_eq!(options.module_names_to_update.len(), 258);
        assert_eq!(options.line_ending, LineEnding::Auto);
        assert_eq!(options.text_changes.comma, SpaceOperation::After);
    }

    #[test]
    fn test_load_or_default_with_missing_file() {
        let options = Options::load_or_default("non_existent_file.toml");
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.uses_section_style, UsesSectionStyle::CommaAtTheEnd);
        assert_eq!(options.override_sorting_order, Vec::<String>::new());
        assert_eq!(options.exclude_files, Vec::<String>::new());
        assert_eq!(options.custom_config_patterns, Vec::<(String, String)>::new());
        assert!(!options.module_names_to_update.is_empty());
        assert_eq!(options.module_names_to_update.len(), 258);
        assert_eq!(options.line_ending, LineEnding::Auto);
        assert_eq!(options.text_changes.comma, SpaceOperation::After);
    }

    #[test]
    fn test_save_and_load() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("test_config.toml");

        let original_options = Options {
            indentation: "    ".to_string(), // 4 spaces
            uses_section_style: UsesSectionStyle::CommaAtTheBeginning,
            override_sorting_order: vec!["test_error".to_string()],
            module_names_to_update: Vec::new(),
            exclude_files: vec!["*.tmp".to_string(), "backup/*".to_string()],
            custom_config_patterns: vec![("test/*.pas".to_string(), "test_config.toml".to_string())],
            line_ending: LineEnding::Lf,
            transformations: TransformationOptions::default(),
            text_changes: TextChangeOptions {
                comma: SpaceOperation::NoChange,
                semi_colon: SpaceOperation::After,
                trim_trailing_whitespace: true,
                ..Default::default()
            },
        };

        // Save options
        original_options.save_to_file(&file_path).unwrap();

        // Load options
        let loaded_options = Options::load_from_file(&file_path).unwrap();

        // ...existing code...
        assert_eq!(loaded_options.indentation, "    ");
        assert_eq!(
            loaded_options.uses_section_style,
            UsesSectionStyle::CommaAtTheBeginning
        );
        assert_eq!(
            loaded_options.override_sorting_order,
            vec!["test_error".to_string()]
        );
        assert_eq!(loaded_options.module_names_to_update, Vec::<String>::new());
        assert_eq!(loaded_options.exclude_files, vec!["*.tmp".to_string(), "backup/*".to_string()]);
        assert_eq!(loaded_options.custom_config_patterns, vec![("test/*.pas".to_string(), "test_config.toml".to_string())]);
        assert_eq!(loaded_options.line_ending, LineEnding::Lf);
        assert_eq!(loaded_options.text_changes.comma, SpaceOperation::NoChange);
        // Manual cleanup
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_partial_toml_file() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("partial_config.toml");

        // Create a TOML file with only some fields set
        fs::write(
            &file_path,
            r#"
# Partial config file with only indentation and line_ending set
indentation = "    "
line_ending = "Lf"
"#,
        )
        .unwrap();

        // This should now parse successfully using defaults for missing fields
        let options = Options::load_from_file(&file_path).unwrap();
        assert_eq!(options.indentation, "    "); // From file
        assert_eq!(options.uses_section_style, UsesSectionStyle::CommaAtTheEnd); // Default
        assert_eq!(options.override_sorting_order, Vec::<String>::new()); // Default
        assert!(!options.module_names_to_update.is_empty()); // Default
        assert_eq!(options.module_names_to_update.len(), 258); // Default
        assert_eq!(options.line_ending, LineEnding::Lf); // From file

        // Clean up
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_empty_toml_file() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("empty_config.toml");

        // Create an empty TOML file
        fs::write(&file_path, "").unwrap();

        // This should parse successfully using all defaults
        let options = Options::load_from_file(&file_path).unwrap();
        let default_options = Options::default();
        assert_eq!(options.indentation, default_options.indentation);
        assert_eq!(
            options.uses_section_style,
            default_options.uses_section_style
        );
        assert_eq!(
            options.override_sorting_order,
            default_options.override_sorting_order
        );
        assert_eq!(
            options.module_names_to_update.len(),
            default_options.module_names_to_update.len()
        );
        assert_eq!(options.line_ending, default_options.line_ending);

        // Clean up
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_partial_transformations_config() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("partial_transformations_config.toml");

        // Create a TOML file with partial transformations section
        fs::write(
            &file_path,
            r#"
indentation = "  "

[transformations]
enable_uses_section = false
# Other transformation options should use defaults
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&file_path).unwrap();
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.uses_section_style, UsesSectionStyle::CommaAtTheEnd);
        assert_eq!(options.override_sorting_order, Vec::<String>::new());
        assert!(!options.module_names_to_update.is_empty());
        assert_eq!(options.module_names_to_update.len(), 258);
        assert_eq!(options.line_ending, LineEnding::Auto);
        assert_eq!(options.text_changes.comma, SpaceOperation::After);
    }

    #[test]
    fn test_line_ending_to_string() {
        assert_eq!(LineEnding::Lf.to_string(), "\n");
        assert_eq!(LineEnding::Crlf.to_string(), "\r\n");

        // Test Auto - it should match OS default
        #[cfg(windows)]
        assert_eq!(LineEnding::Auto.to_string(), "\r\n");
        #[cfg(not(windows))]
        assert_eq!(LineEnding::Auto.to_string(), "\n");
    }

    #[test]
    fn test_should_exclude_file() {
        // Test with no exclusion patterns
        let empty_patterns = vec![];
        assert!(!should_exclude_file(&empty_patterns, "test.pas", None));

        // Test with single pattern
        let single_pattern = vec!["*.tmp".to_string()];
        assert!(should_exclude_file(&single_pattern, "test.tmp", None));
        assert!(!should_exclude_file(&single_pattern, "test.pas", None));

        // Test with multiple patterns
        let multiple_patterns = vec![
            "*.tmp".to_string(),
            "test/*".to_string(),
            "backup*.pas".to_string(),
        ];
        assert!(should_exclude_file(&multiple_patterns, "file.tmp", None));
        assert!(should_exclude_file(&multiple_patterns, "test/file.pas", None));
        assert!(should_exclude_file(&multiple_patterns, "backup_old.pas", None));
        assert!(!should_exclude_file(&multiple_patterns, "normal.pas", None));

        // Test with path normalization
        assert!(should_exclude_file(&multiple_patterns, "test\\file.pas", None));
    }

    #[test]
    fn test_should_exclude_file_with_config_path() {
        let patterns = vec!["test/*.pas".to_string()];

        // Test relative to config directory
        let config_path = "project/dfixxer.toml";
        assert!(should_exclude_file(&patterns, "test/file.pas", Some(config_path)));
        assert!(!should_exclude_file(&patterns, "src/file.pas", Some(config_path)));
    }

    #[test]
    fn test_invalid_glob_pattern() {
        // Invalid pattern should be ignored (not crash)
        let invalid_patterns = vec!["[invalid".to_string()];

        // Should not match anything due to invalid pattern
        assert!(!should_exclude_file(&invalid_patterns, "test.pas", None));
    }

    #[test]
    fn test_line_ending_direct_usage() {
        let mut options = Options::default();

        options.line_ending = LineEnding::Lf;
        assert_eq!(options.line_ending.to_string(), "\n");

        options.line_ending = LineEnding::Crlf;
        assert_eq!(options.line_ending.to_string(), "\r\n");

        options.line_ending = LineEnding::Auto;
        #[cfg(windows)]
        assert_eq!(options.line_ending.to_string(), "\r\n");
        #[cfg(not(windows))]
        assert_eq!(options.line_ending.to_string(), "\n");
    }

    #[test]
    fn test_config_with_exclude_files() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("config_with_excludes.toml");

        // Create a TOML file with exclude_files
        fs::write(
            &file_path,
            r#"
indentation = "  "
exclude_files = ["*.tmp", "backup/*", "test_*.pas"]

[transformations]
enable_uses_section = true
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&file_path).unwrap();
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.exclude_files.len(), 3);
        assert_eq!(options.exclude_files[0], "*.tmp");
        assert_eq!(options.exclude_files[1], "backup/*");
        assert_eq!(options.exclude_files[2], "test_*.pas");

        // Clean up
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_find_custom_config_for_file() {
        // Test with no custom patterns
        let empty_patterns = vec![];
        assert!(find_custom_config_for_file(&empty_patterns, "test.pas", None).is_none());

        // Test with single pattern match
        let single_pattern = vec![("test/*.pas".to_string(), "custom.toml".to_string())];
        let result = find_custom_config_for_file(&single_pattern, "test/file.pas", Some("project/dfixxer.toml"));
        // Normalize path separators for cross-platform compatibility
        let expected = Path::new("project").join("custom.toml").to_string_lossy().to_string();
        assert_eq!(result, Some(expected));

        // Test with absolute path
        let absolute_pattern = vec![("test/*.pas".to_string(), "/absolute/custom.toml".to_string())];
        let result = find_custom_config_for_file(&absolute_pattern, "test/file.pas", Some("project/dfixxer.toml"));
        assert_eq!(result, Some("/absolute/custom.toml".to_string()));

        // Test with no match
        let no_match_pattern = vec![("other/*.pas".to_string(), "custom.toml".to_string())];
        let result = find_custom_config_for_file(&no_match_pattern, "test/file.pas", Some("project/dfixxer.toml"));
        assert!(result.is_none());

        // Test with multiple patterns
        let multiple_patterns = vec![
            ("test/*.pas".to_string(), "test_custom.toml".to_string()),
            ("src/*.pas".to_string(), "src_custom.toml".to_string()),
            ("backup*.pas".to_string(), "backup_custom.toml".to_string()),
        ];
        let result = find_custom_config_for_file(&multiple_patterns, "src/main.pas", Some("project/dfixxer.toml"));
        let expected = Path::new("project").join("src_custom.toml").to_string_lossy().to_string();
        assert_eq!(result, Some(expected));

        // Test without base config path
        let result = find_custom_config_for_file(&single_pattern, "test/file.pas", None);
        assert_eq!(result, Some("custom.toml".to_string()));
    }

    #[test]
    fn test_custom_config_patterns_serialization() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("custom_patterns_config.toml");

        // Create a TOML file with custom_config_patterns
        fs::write(
            &file_path,
            r#"
indentation = "  "
custom_config_patterns = [
    ["test/*.pas", "test_config.toml"],
    ["src/**/*.pas", "../src/dfixxer.toml"],
    ["legacy/*.pas", "/absolute/legacy_config.toml"]
]

[transformations]
enable_uses_section = true
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&file_path).unwrap();
        assert_eq!(options.custom_config_patterns.len(), 3);
        assert_eq!(options.custom_config_patterns[0], ("test/*.pas".to_string(), "test_config.toml".to_string()));
        assert_eq!(options.custom_config_patterns[1], ("src/**/*.pas".to_string(), "../src/dfixxer.toml".to_string()));
        assert_eq!(options.custom_config_patterns[2], ("legacy/*.pas".to_string(), "/absolute/legacy_config.toml".to_string()));

        // Clean up
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_config_loading_with_different_line_endings() {
        // Test loading config with Auto
        let temp_path = create_unique_temp_dir();
        let auto_config_path = temp_path.join("auto_config.toml");
        fs::write(
            &auto_config_path,
            r#"
indentation = "  "
uses_section_style = "CommaAtTheEnd"
override_sorting_order = []
module_names_to_update = []
line_ending = "Auto"

[transformations]
enable_uses_section = true
enable_unit_program_section = true
enable_single_keyword_sections = true
enable_procedure_section = true
enable_text_transformations = true

[text_changes]
comma = "After"
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&auto_config_path).unwrap();
        assert_eq!(options.line_ending, LineEnding::Auto);

        // Test loading config with Lf
        let lf_config_path = temp_path.join("lf_config.toml");
        fs::write(
            &lf_config_path,
            r#"
indentation = "  "
uses_section_style = "CommaAtTheEnd"
override_sorting_order = []
module_names_to_update = []
line_ending = "Lf"

[transformations]
enable_uses_section = true
enable_unit_program_section = true
enable_single_keyword_sections = true
enable_procedure_section = true
enable_text_transformations = true

[text_changes]
comma = "NoChange"
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&lf_config_path).unwrap();
        assert_eq!(options.line_ending, LineEnding::Lf);

        // Test loading config with Crlf
        let crlf_config_path = temp_path.join("crlf_config.toml");
        fs::write(
            &crlf_config_path,
            r#"
indentation = "  "
uses_section_style = "CommaAtTheEnd"
override_sorting_order = []
module_names_to_update = []
line_ending = "Crlf"

[transformations]
enable_uses_section = true
enable_unit_program_section = true
enable_single_keyword_sections = true
enable_procedure_section = true
enable_text_transformations = true

[text_changes]
comma = "After"
"#,
        )
        .unwrap();

        let options = Options::load_from_file(&crlf_config_path).unwrap();
        assert_eq!(options.line_ending, LineEnding::Crlf);

        // Clean up
        fs::remove_file(&auto_config_path).ok();
        fs::remove_file(&lf_config_path).ok();
        fs::remove_file(&crlf_config_path).ok();
        fs::remove_dir(&temp_path).ok();
    }
}
