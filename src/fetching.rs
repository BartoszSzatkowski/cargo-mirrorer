#[derive(Debug)]
pub enum FetchPlan {
    AllCrates,
    Playground,
    TopN(usize),
    List(Vec<String>),
}

impl TryFrom<String> for FetchPlan {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &value {
            "all" => Ok(Self::AllCrates),
            "playground" => Ok(Self::Playground),
            n if n.parse::<usize>().is_ok() => Ok(Self::TopN(n.parse::<usize>().unwrap())),
            l if l.contains(',') => Ok(Self::List(l.split(",").map(String::from).collect())),
            _ => Err("Invalid fetching strategy provided".to_string()),
        }
    }
}
