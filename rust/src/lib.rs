use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::join_all;
use tokio::{sync::Mutex, time};

mod utils;
pub use utils::{Error, XnodeControllerError};

use crate::utils::{XnodeControllerErrorInner, add_user_config, outside};

pub trait XnodeController: Send + Sync {
    /// Get session to update Xnode
    fn get_session(&self) -> &xnode_manager_sdk::utils::Session;

    /// Decide who should be the current controller based on external data
    fn check_controller(&self) -> impl Future<Output = Option<String>> + Send;

    // What OS config block should be set for the controller
    fn controller_config(&self, controller: String) -> String;

    // How to uniquely identify each Xnode
    fn xnode_identifier(&self) -> String {
        self.get_session().base_url.clone()
    }

    /// Set a new owner
    fn set_controller(
        &self,
        controller: Option<String>,
    ) -> impl Future<Output = Result<xnode_manager_sdk::os::SetOutput, Error>> + Send {
        async {
            let session = self.get_session();
            let xnode_id = self.xnode_identifier();
            let current_os =
                xnode_manager_sdk::os::get(xnode_manager_sdk::os::GetInput::new(session))
                    .await
                    .map_err(Error::XnodeManagerSDKError)?;

            let new_controller_config = if let Some(controller) = controller {
                self.controller_config(controller)
            } else {
                "".to_string()
            };
            let os_block_start = format!("# START XNODE CONTROLLER {}", xnode_id);
            let os_block_end = format!("# END XNODE CONTROLLER {}", xnode_id);
            let new_os_config = match outside(&current_os.flake, &os_block_start, &os_block_end) {
                // Update existing xnode controller block
                Some((before, after)) => [before, &new_controller_config, after].join("\n"),
                // No xnode controller block, insert at start of user config
                None => add_user_config(
                    &current_os.flake,
                    "# START USER CONFIG",
                    &[
                        "",
                        &os_block_start,
                        &new_controller_config,
                        &os_block_end,
                        "",
                    ]
                    .join("\n"),
                )
                .ok_or(Error::XnodeControllerError(XnodeControllerError::new(
                    XnodeControllerErrorInner::NoUserConfig,
                )))?,
            };
            let os_update =
                xnode_manager_sdk::os::set(xnode_manager_sdk::os::SetInput::new_with_data(
                    session,
                    xnode_manager_sdk::os::OSChange {
                        acme_email: None,
                        domain: None,
                        flake: Some(new_os_config),
                        update_inputs: None,
                        user_passwd: None,
                        xnode_owner: None,
                    },
                ))
                .await
                .map_err(Error::XnodeManagerSDKError)?;

            Ok(os_update)
        }
    }
}

struct XnodeControllerData {
    last_controller: Option<String>,
}
pub async fn update_controllers<Xnode: XnodeController + 'static>(
    xnodes: Arc<Mutex<Vec<Xnode>>>,
    update_interval: Duration,
) {
    let mut interval = time::interval(update_interval);
    let data = Arc::new(Mutex::new(HashMap::<String, XnodeControllerData>::new()));

    loop {
        interval.tick().await;

        join_all(xnodes.lock().await.iter().map(async |xnode| {
            let xnode_id = xnode.xnode_identifier();
            let new_controller = xnode.check_controller().await;
            if let Some(data) = data.lock().await.get(&xnode_id) {
                if new_controller == data.last_controller {
                    return;
                }
            }

            log::info!("Setting controller on {xnode_id} to {new_controller:?}");
            if let Err(e) = xnode.set_controller(new_controller.clone()).await {
                log::warn!("Error setting controller on {xnode_id} to {new_controller:?}: {e:?}");
            }
            data.lock()
                .await
                .entry(xnode_id)
                .and_modify(|data| {
                    data.last_controller = new_controller.clone();
                })
                .or_insert(XnodeControllerData {
                    last_controller: new_controller,
                });
        }))
        .await;
    }
}
