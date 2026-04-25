#[derive(Debug)]
pub struct Method(MethodInner);

#[derive(Debug)]
pub enum MethodInner {
    Get,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            MethodInner::Get => write!(f, "GET"),
        }
    }
}

impl Method {
    pub const GET: Self = Self(MethodInner::Get);
}
