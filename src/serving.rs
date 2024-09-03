#[derive(Debug, PartialEq)]
pub enum ServingPlan {
    Git,
    Sparse,
}

impl TryFrom<String> for ServingPlan {
    type Error = String;

    fn try_from(value: String) -> anyhow::Result<Self, Self::Error> {
        match value.as_str() {
            "git" => Ok(Self::Git),
            "sparse" => Ok(Self::Sparse),
            _ => Err("Invalid serving strategy provided".to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_parses_serving_flag_correctly() {
        assert_eq!(
            Ok(ServingPlan::Git),
            ServingPlan::try_from("git".to_string())
        );
        assert_eq!(
            Ok(ServingPlan::Sparse),
            ServingPlan::try_from("sparse".to_string())
        );
    }
}
