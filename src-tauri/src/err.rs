use std::fmt::Display;

#[derive(Debug)]
#[allow(unused)]
pub enum Err {
  Parse{pos: usize},
  Eval(),
  Num(),
}

impl Display for Err {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Err::Parse{pos} => {
        f.write_str("Err::Parse{pos: ")?;
        pos.fmt(f)?;
        f.write_str("}")?;
      },
      Err::Eval() => f.write_str("Err::Eval")?,
      Err::Num() => f.write_str("Err::Num")?,
    };
    Ok(())
  }
}

impl std::error::Error for Err {
  fn cause(&self) -> Option<&dyn std::error::Error> {
    self.source()
  }
  fn description(&self) -> &str {
    &"ValuatorError"
  }
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_err_basics() {
    assert_eq!("Err::Parse{pos: 0}", Err::Parse { pos: 0 }.to_string())
  }
}
