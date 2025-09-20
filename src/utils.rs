use anyhow::Result;

pub fn is_python_312_or_greater(version: &str) -> Result<bool> {
    let mut split_version = version.split('.');
    if let Some(v) = split_version.nth(1) {
        let min = v.parse::<i32>()?;
        if min >= 12 {
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_312() {
        let result = is_python_312_or_greater("3.12").unwrap();
        assert!(result);
    }

    #[test]
    fn test_python_313() {
        let result = is_python_312_or_greater("3.13").unwrap();
        assert!(result);
    }

    #[test]
    fn test_python_311() {
        let result = is_python_312_or_greater("3.11").unwrap();
        assert!(!result);
    }
}
