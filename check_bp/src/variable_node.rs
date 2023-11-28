use belief_propagation::{BPError, BPResult, Msg, NodeFunction, NodeIndex};

use crate::bin_var_node::{CtrlMsgA, CtrlMsg};

#[derive(Clone)]
pub struct VariableNode<T, MsgT: Msg<T>> {
    //TODO:
    connections: Option<Vec<NodeIndex>>,
    prior: Option<MsgT>,
    has_propagated: bool,
    phantom: std::marker::PhantomData<T>,
}

impl<T, MsgT: Msg<T>> VariableNode<T, MsgT>
where
    MsgT: Clone,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        VariableNode {
            connections: None,
            prior: None,
            has_propagated: false,
            phantom: std::marker::PhantomData,
        }
    }
    #[allow(dead_code)]
    pub fn set_prior(&mut self, prior: &MsgT) -> BPResult<()> {
        if self.prior.is_some() {
            return Err(BPError::new(
                "VariableNode::set_prior".to_owned(),
                "Prior is already set".to_owned(),
            ));
        }
        self.prior = Some(prior.clone());
        Ok(())
    }
}

impl<T, MsgT: Msg<T>> NodeFunction<T, MsgT, CtrlMsg, CtrlMsgA> for VariableNode<T, MsgT>
where
    MsgT: Clone,
{
    fn is_ready(&self, recv_from: &Vec<(NodeIndex, MsgT)>, _step: usize) -> BPResult<bool> {
        Ok(
            if recv_from.len()
                == self
                    .connections
                    .as_ref()
                    .expect("Node not initialized.")
                    .len()
                || !self.has_propagated
            {
                true
            } else {
                false
            }
        )
    }

    fn get_prior(&self) -> Option<MsgT> {
        self.prior.clone()
    }

    fn initialize(&mut self, connections: Vec<NodeIndex>) -> BPResult<()> {
        self.connections = Some(connections);
        Ok(())
    }

    fn node_function(
        &mut self,
        mut inbox: Vec<(NodeIndex, MsgT)>,
    ) -> BPResult<Vec<(NodeIndex, MsgT)>> {
        let connections = self
            .connections
            .as_ref()
            .expect("VariableNode not initialized");
        self.has_propagated = true;
        if inbox.is_empty() {
            if let Some(prior) = &self.prior {
                Ok(connections
                    .iter()
                    .map(|idx| (*idx, prior.clone()))
                    .collect())
            } else {
                Err(BPError::new(
                    "VariableNode::node_function".to_owned(),
                    "Inbox is empty".to_owned(),
                ))
            }
        } else if inbox.len() == 1 {
            let (idx_in, mut msg_in) = inbox.pop().unwrap();
            let mut out: Vec<(NodeIndex, MsgT)> = Vec::new();
            if let Some(prior) = &self.prior {
                msg_in.mult_msg(prior);
                out.push((idx_in, prior.clone()));
            }
            for con in connections {
                if idx_in != *con {
                    out.push((*con, msg_in.clone()));
                }
            }
            Ok(out)
        } else if inbox.len() == connections.len(){
            let mut result: Vec<(NodeIndex, MsgT)> = Vec::with_capacity(inbox.len());
            let n = inbox.len();
            let (mut acc, start) = if let Some(prior) = &self.prior {
                (prior.clone(), 0)
            } else {
                (inbox[0].1.clone(), 1)
            };
            for msg in &inbox[start..] {
                result.push((msg.0, acc.clone()));
                acc.mult_msg(&msg.1);
            }
            acc = inbox[n - 1].1.clone();
            for idx in (0..n - 1 - start).rev() {
                result[idx].1.mult_msg(&acc);
                acc.mult_msg(&inbox[idx + start].1);
            }
            if start == 1 {
                result.push((inbox[0].0, acc.clone()));
            }
            Ok(result)
        } else {
            let mut result: Vec<(NodeIndex, MsgT)> = Vec::with_capacity(connections.len());
            let mut missing = connections.clone();
            let n = inbox.len();
            let (mut acc, start) = if let Some(prior) = &self.prior {
                (prior.clone(), 0)
            } else {
                missing.retain(|idx| *idx != inbox[0].0);
                (inbox[0].1.clone(), 1)
            };
            for msg in &inbox[start..] {
                result.push((msg.0, acc.clone()));
                acc.mult_msg(&msg.1);
                missing.retain(|idx| *idx != msg.0);
            }
            acc = inbox[n - 1].1.clone();
            for idx in (0..n - 1 - start).rev() {
                result[idx].1.mult_msg(&acc);
                acc.mult_msg(&inbox[idx + start].1);
            }
            if start == 1 {
                result.push((inbox[0].0, acc.clone()));
                acc.mult_msg(&inbox[0].1);
            }
            assert_eq!(missing.len() + result.len(), connections.len());
            for idx in missing {
                result.push((idx, acc.clone()));
            }
            Ok(result)
        }
    }

    fn reset(&mut self) -> BPResult<()> {
        self.prior = None;
        Ok(())
    }

    fn is_factor(&self) -> bool {
        false
    }

    fn number_inputs(&self) -> Option<usize> {
        None
    }
    fn send_control_message(&mut self, _ctrl_msg: CtrlMsg) -> BPResult<CtrlMsgA> {
        panic!("Not implemented");
        /*
        Ok(match ctrl_msg {
            CtrlMsg::GetFixed => CtrlMsgA::Fixed(self.is_fixed.is_some()),
            CtrlMsg::SetFixed(v) => {
                assert!(v >= -(ETA as i16) / 2 && v <= ETA as i16 / 2);
                self.set_fixed(Some(v));
                CtrlMsgA::default()
            }
        })
        */
    }
}

impl<T, MsgT: Msg<T> + Clone> Default for VariableNode<T, MsgT> {
    fn default() -> Self {
        Self::new()
    }
}
