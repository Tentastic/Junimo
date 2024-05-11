use std::path::PathBuf;

pub fn app_path(file: &str) -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push("Junimo");
    path.push(file);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_path_test() {
        let mut test_dir = dirs::config_dir().unwrap();
        test_dir.push("Junimo");
        test_dir.push("test.txt");

        let result = app_path("test.txt");
        assert_eq!(result, test_dir);
    }
}
