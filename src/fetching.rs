#[derive(Debug, PartialEq)]
pub enum FetchPlan {
    AllCrates,
    Playground,
    TopN(usize),
    List(Vec<String>),
}

impl TryFrom<String> for FetchPlan {
    type Error = String;

    fn try_from(value: String) -> anyhow::Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(Self::AllCrates),
            "playground" => Ok(Self::Playground),
            n if n.parse::<usize>().is_ok() => Ok(Self::TopN(n.parse::<usize>().unwrap())),
            l if l.contains(',') => Ok(Self::List(l.split(",").map(String::from).collect())),
            _ => Err("Invalid fetching strategy provided".to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_parses_flag_correctly() {
        assert_eq!(
            Ok(FetchPlan::AllCrates),
            FetchPlan::try_from("all".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::Playground),
            FetchPlan::try_from("playground".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::TopN(100)),
            FetchPlan::try_from("100".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::List(vec![
                "tokio".to_string(),
                "serde".to_string()
            ])),
            FetchPlan::try_from("tokio,serde".to_string())
        );
        assert_eq!(
            Err("Invalid fetching strategy provided".to_string()),
            FetchPlan::try_from("v9W1x".to_string())
        );
    }
}
