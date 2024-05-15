use winreg::enums::HKEY_CLASSES_ROOT;
use winreg::RegKey;

pub trait RegistryManager {
    fn create_subkey(&self, key: &str) -> Result<Box<dyn RegistryManager>, String>;
    fn set_value(&self, key: &str, value: &str) -> Result<(), String>;
}

pub struct RealRegistryManager {
    reg_key: RegKey,
}

impl RealRegistryManager {
    pub fn new() -> Result<Self, String> {
        let hklm = RegKey::predef(HKEY_CLASSES_ROOT);
        Ok(RealRegistryManager { reg_key: hklm })
    }
}

impl RegistryManager for RealRegistryManager {
    fn create_subkey(&self, key: &str) -> Result<Box<dyn RegistryManager>, String> {
        self.reg_key
            .create_subkey(key)
            .map(|(key, _)| {
                Box::new(RealRegistryManager { reg_key: key }) as Box<dyn RegistryManager>
            })
            .map_err(|e| e.to_string())
    }

    fn set_value(&self, key: &str, value: &str) -> Result<(), String> {
        self.reg_key
            .set_value(key, &value.to_string())
            .map_err(|e| e.to_string())
    }
}

pub struct MockRegistryManager;
impl RegistryManager for MockRegistryManager {
    fn create_subkey(&self, key: &str) -> Result<Box<dyn RegistryManager>, String> {
        Ok(Box::new(MockRegistryManager))
    }

    fn set_value(&self, key: &str, value: &str) -> Result<(), String> {
        Ok(())
    }
}
