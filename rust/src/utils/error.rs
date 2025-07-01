use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    XnodeControllerError(XnodeControllerError),
    XnodeManagerSDKError(xnode_manager_sdk::utils::Error),
}

#[derive(Debug)]
pub struct XnodeControllerError {
    error: Box<XnodeControllerErrorInner>,
}

impl Display for XnodeControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.error.as_ref() {
            XnodeControllerErrorInner::NoUserConfig => "No User Config",
        })
    }
}

#[derive(Debug)]
pub enum XnodeControllerErrorInner {
    NoUserConfig,
}

impl XnodeControllerError {
    pub fn new(error: XnodeControllerErrorInner) -> Self {
        Self {
            error: Box::new(error),
        }
    }
}
