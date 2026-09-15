//! Graph-based Intermediate Representation
//! 
//! Unlike traditional SSA-based IRs, Chronos uses a graph representation
//! that can be processed by Graph Neural Networks for optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphIR {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
    pub entry: NodeId,
    pub exit: NodeId,
    next_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub op: Operation,
    pub inputs: Vec<NodeId>,
    pub outputs: Vec<NodeId>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logic
    And, Or, Not,
    // Control flow
    Branch { condition: NodeId },
    Jump { target: NodeId },
    Return,
    // Memory
    Load { addr: NodeId },
    Store { addr: NodeId, value: NodeId },
    Alloca { size: u64 },
    // Function
    Call { func: String, args: Vec<NodeId> },
    // Constants
    ConstInt(i64),
    ConstFloat(f64),
    // LLM-generated
    LlmGenerated { prompt: String, code: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub estimated_cost: f64,
    pub is_hot: bool,
    pub llm_confidence: f32,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeKind {
    DataFlow,
    ControlFlow,
    SideEffect,
}

impl GraphIR {
    pub fn new() -> Self {
        let mut ir = Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            entry: NodeId(0),
            exit: NodeId(0),
            next_id: 0,
        };
        ir.entry = ir.add_node(Operation::Jump { target: NodeId(0) }, vec![]);
        ir.exit = ir.add_node(Operation::Return, vec![]);
        ir
    }

    pub fn add_node(&mut self, op: Operation, inputs: Vec<NodeId>) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        // Add data flow edges
        for input in &inputs {
            self.edges.push(Edge {
                from: *input,
                to: id,
                kind: EdgeKind::DataFlow,
            });
        }

        self.nodes.insert(id, Node {
            id,
            op,
            inputs: inputs.clone(),
            outputs: vec![],
            metadata: NodeMetadata {
                estimated_cost: 1.0,
                is_hot: false,
                llm_confidence: 1.0,
                verified: false,
            },
        });

        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) {
        self.edges.push(Edge { from, to, kind });
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Convert to adjacency matrix for GNN processing
    pub fn to_adjacency_matrix(&self) -> ndarray::Array2<f32> {
        let n = self.nodes.len();
        let mut matrix = ndarray::Array2::zeros((n, n));

        for edge in &self.edges {
            let from_idx = edge.from.0 as usize;
            let to_idx = edge.to.0 as usize;
            if from_idx < n && to_idx < n {
                matrix[[from_idx, to_idx]] = 1.0;
            }
        }

        matrix
    }

    /// Extract feature matrix for GNN
    pub fn to_feature_matrix(&self) -> ndarray::Array2<f32> {
        let n = self.nodes.len();
        let feature_dim = 16;
        let mut matrix = ndarray::Array2::zeros((n, feature_dim));

        for (i, node) in self.nodes.values().enumerate() {
            let features = self.node_features(node);
            for (j, &f) in features.iter().enumerate() {
                if j < feature_dim {
                    matrix[[i, j]] = f;
                }
            }
        }

        matrix
    }

    fn node_features(&self, node: &Node) -> Vec<f32> {
        let mut features = vec![0.0; 16];

        // Opcode one-hot (first 10 dimensions)
        let op_idx = match &node.op {
            Operation::Add => 0,
            Operation::Sub => 1,
            Operation::Mul => 2,
            Operation::Div => 3,
            Operation::Branch { .. } => 4,
            Operation::Call { .. } => 5,
            Operation::Load { .. } => 6,
            Operation::Store { .. } => 7,
            Operation::LlmGenerated { .. } => 8,
            _ => 9,
        };
        features[op_idx] = 1.0;

        // Metadata features
        features[10] = node.metadata.estimated_cost as f32;
        features[11] = if node.metadata.is_hot { 1.0 } else { 0.0 };
        features[12] = node.metadata.llm_confidence;
        features[13] = if node.metadata.verified { 1.0 } else { 0.0 };

        // Degree features
        features[14] = node.inputs.len() as f32;
        features[15] = node.outputs.len() as f32;

        features
    }
}
