use crate::bin_tree::BinMultTree;
use crate::check_msg::CheckMsg;
use belief_propagation::{BPError, BPResult, Msg, NodeFunction, NodeIndex};

pub enum CtrlMsgA {
    Fixed(bool),
    None,
}

pub enum CtrlMsg {
    SetFixed(i16),
    GetFixed,
}

impl std::default::Default for CtrlMsgA {
    fn default() -> Self {
        Self::None
    }
}

fn mult_msgs<const ETA: usize>(op0: &CheckMsg<ETA>, op1: &CheckMsg<ETA>) -> CheckMsg<ETA> {
    let mut data = [0 as f64; ETA];
    for (i, (p0, p1)) in op0.data.iter().zip(op1.data.iter()).enumerate() {
        data[i] = p0 * p1;
    }
    let mut res = CheckMsg::from_data(data);
    res.normalize().expect("Failed to normalize.");
    res
}

#[derive(Clone)]
pub struct BinVariableNode<const ETA: usize> {
    connections: Option<Vec<NodeIndex>>,
    prior: Option<CheckMsg<ETA>>,
    has_propagated: bool,
    is_fixed: Option<i16>,
}

impl<const ETA: usize> BinVariableNode<ETA> {
    pub fn new() -> Self {
        BinVariableNode {
            prior: None,
            connections: None,
            has_propagated: false,
            is_fixed: None,
        }
    }

    pub fn set_fixed(&mut self, value: Option<i16>) {
        self.is_fixed = value;
    }

    pub fn set_prior(&mut self, prior: &CheckMsg<ETA>) -> BPResult<()> {
        if self.prior.is_some() {
            return Err(BPError::new(
                "BinVariableNode::set_prior".to_owned(),
                "Prior is already set.".to_owned(),
            )
            .attach_debug_object("prior (the prior set before)", &prior));
        }
        self.prior = Some(*prior);
        Ok(())
    }
}

impl<const ETA: usize> NodeFunction<i16, CheckMsg<ETA>, CtrlMsg, CtrlMsgA>
    for BinVariableNode<ETA>
{
    fn get_prior(&self) -> Option<CheckMsg<ETA>> {
        self.prior.clone()
    }
    fn initialize(&mut self, connections: Vec<NodeIndex>) -> BPResult<()> {
        if self.prior.is_none() {
            return Err(BPError::new(
                "BinVariableNode::initialize".to_owned(),
                "BinVariableNode expects a prior".to_owned(),
            ));
        }
        self.connections = Some(connections);
        Ok(())
    }
    fn node_function(
        &mut self,
        inbox: Vec<(NodeIndex, CheckMsg<ETA>)>,
    ) -> BPResult<Vec<(NodeIndex, CheckMsg<ETA>)>> {
        if let Some(v) = self.is_fixed {
            let mut msg = CheckMsg::new();
            msg.set_fixed_value(v);
            return Ok(self
                .connections
                .as_ref()
                .expect("Did not find connections.")
                .iter()
                .map(|con| (*con, msg))
                .collect());
        }
        if !self.has_propagated {
            self.has_propagated = true;
            return Ok(self
                .connections
                .as_ref()
                .expect("Did not find connections. Node not initialized?")
                .iter()
                .map(|con| (*con, self.prior.expect("No prior set").clone()))
                .collect());
        }
        let mut leafs = Vec::with_capacity(inbox.len());
        let mut indices = Vec::with_capacity(inbox.len());
        for (index, msg) in inbox {
            if !msg.is_valid() {
                return Err(BPError::new(
                    "BinVariableNode::node_function".to_owned(),
                    "Invalid message in inbox".to_owned(),
                )
                .attach_debug_object("msg (the invalid message)", msg)
                .attach_debug_object("index (Index of the node the msg came from)", index));
            }
            leafs.push(msg);
            indices.push(index);
        }
        let prods = BinMultTree::new(leafs, mult_msgs)
            .calculate_with_prior(&self.prior.expect("No prior set."));
        for msg in &prods {
            if !msg.is_valid() {
                return Err(BPError::new(
                    "BinVariableNode::node_function".to_owned(),
                    "Invalid message produced by BinMultTree".to_owned(),
                )
                .attach_debug_object("msg (the invalid message)", msg));
            }
        }
        Ok(indices.into_iter().zip(prods.into_iter()).collect())
    }

    fn reset(&mut self) -> BPResult<()> {
        self.prior = None;
        Ok(())
    }
    fn is_factor(&self) -> bool {
        false
    }
    fn number_inputs(&self) -> Option<usize> {
        self.connections.clone().map(|c| c.len())
    }
    fn send_control_message(&mut self, ctrl_msg: CtrlMsg) -> BPResult<CtrlMsgA> {
        Ok(match ctrl_msg {
            CtrlMsg::GetFixed => CtrlMsgA::Fixed(self.is_fixed.is_some()),
            CtrlMsg::SetFixed(v) => {
                assert!(v >= -(ETA as i16) / 2 && v <= ETA as i16 / 2);
                self.set_fixed(Some(v));
                CtrlMsgA::default()
            }
        })
    }
    fn is_ready(
        &self,
        recv_from: &Vec<(NodeIndex, CheckMsg<ETA>)>,
        current_step: usize,
    ) -> BPResult<bool> {
        if let Some(connections) = &self.connections {
            Ok(recv_from.len() == connections.len() || current_step == 0)
        } else {
            Err(BPError::new(
                "BinVariableNode::is_ready".to_owned(),
                "Node is not initialized".to_owned(),
            ))
        }
    }
}
