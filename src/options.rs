use crate::dfixxer_error::DFixxerError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
pub struct TransformationOptions {
    pub enable_uses_section: bool,
    pub enable_unit_program_section: bool,
    pub enable_single_keyword_sections: bool,
    pub enable_procedure_section: bool,
}

impl Default for TransformationOptions {
    fn default() -> Self {
        TransformationOptions {
            enable_uses_section: true,
            enable_unit_program_section: true,
            enable_single_keyword_sections: true,
            enable_procedure_section: true,
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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            indentation: "  ".to_string(),
            uses_section_style: UsesSectionStyle::CommaAtTheEnd,
            override_sorting_order: Vec::new(),
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
        }
    }
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
        assert!(!options.module_names_to_update.is_empty());
        assert_eq!(options.module_names_to_update.len(), 258);
        assert_eq!(options.line_ending, LineEnding::Auto);
    }

    #[test]
    fn test_load_or_default_with_missing_file() {
        let options = Options::load_or_default("non_existent_file.toml");
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.uses_section_style, UsesSectionStyle::CommaAtTheEnd);
        assert_eq!(options.override_sorting_order, Vec::<String>::new());
        assert!(!options.module_names_to_update.is_empty());
        assert_eq!(options.module_names_to_update.len(), 258);
        assert_eq!(options.line_ending, LineEnding::Auto);
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
            line_ending: LineEnding::Lf,
            transformations: TransformationOptions::default(),
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
        assert_eq!(loaded_options.line_ending, LineEnding::Lf);
        // Manual cleanup
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
    }

    #[test]
    fn test_partial_toml_file() {
        let temp_path = create_unique_temp_dir();
        let file_path = temp_path.join("partial_config.toml");

        // Create a TOML file with only some fields set
        fs::write(&file_path, r#"
# Partial config file with only indentation and line_ending set
indentation = "    "
line_ending = "Lf"
"#).unwrap();

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
        assert_eq!(options.uses_section_style, default_options.uses_section_style);
        assert_eq!(options.override_sorting_order, default_options.override_sorting_order);
        assert_eq!(options.module_names_to_update.len(), default_options.module_names_to_update.len());
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
        fs::write(&file_path, r#"
indentation = "  "

[transformations]
enable_uses_section = false
# Other transformation options should use defaults
"#).unwrap();

        let options = Options::load_from_file(&file_path).unwrap();
        assert_eq!(options.indentation, "  ");
        assert_eq!(options.transformations.enable_uses_section, false); // From file
        assert_eq!(options.transformations.enable_unit_program_section, true); // Default
        assert_eq!(options.transformations.enable_single_keyword_sections, true); // Default
        assert_eq!(options.transformations.enable_procedure_section, true); // Default

        // Clean up
        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_path).ok();
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
