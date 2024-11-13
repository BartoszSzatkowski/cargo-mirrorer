use anyhow::anyhow;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum FetchPlan {
    AllCrates,
    Playground,
    TopN(usize),
    List(Vec<String>),
}

impl FromStr for FetchPlan {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::AllCrates),
            "playground" => Ok(Self::Playground),
            n if n.parse::<usize>().is_ok() => Ok(Self::TopN(n.parse::<usize>()?)),
            l if l.contains(',') => Ok(Self::List(l.split(",").map(String::from).collect())),
            _ => Err(anyhow!("Invalid fetching strategy provided")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_parses_fetching_flag_correctly() {
        assert_eq!(FetchPlan::AllCrates, FetchPlan::from_str("all").unwrap());
        assert_eq!(
            FetchPlan::Playground,
            FetchPlan::from_str("playground").unwrap()
        );
        assert_eq!(FetchPlan::TopN(100), FetchPlan::from_str("100").unwrap());
        assert_eq!(
            FetchPlan::List(vec!["tokio".to_string(), "serde".to_string()]),
            FetchPlan::from_str("tokio,serde").unwrap()
        );
        assert!(FetchPlan::from_str("v9W1x").is_err())
    }
}
