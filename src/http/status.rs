use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(u16);

impl FromStr for StatusCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code = s.parse::<u16>()?;
        Ok(StatusCode(code))
    }
}

impl StatusCode {
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }

    pub fn is_redirect(self) -> bool {
        [301, 302, 303, 307, 308].contains(&self.0)
    }
}
