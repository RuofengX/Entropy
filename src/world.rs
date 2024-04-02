use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    active_nodes: Vec<Node>,
    
}
