// Copyright (c) 2018 Pietro Albini <pietro@pietroalbini.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use cargo_metadata::Resolve;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    graph: DiGraph<String, ()>,
    crates: HashMap<String, NodeIndex>,
    root: NodeIndex,
}

impl DependencyGraph {
    pub fn new() -> Self {
        let mut graph = DiGraph::new();
        let root = graph.add_node(String::new());

        DependencyGraph {
            graph,
            crates: HashMap::new(),
            root,
        }
    }

    pub fn load_from_metadata(&mut self, resolve: &Resolve) {
        // First load all the crates
        for node in &resolve.nodes {
            let name = node.id.split(' ').next().unwrap().to_string();
            if self.crates.contains_key(&name) {
                continue;
            }

            let id = if name.starts_with("dummy-") {
                self.root
            } else {
                self.graph.add_node(name.clone())
            };

            self.crates.insert(name, id);
        }

        // Then connect all the dependencies
        for node in &resolve.nodes {
            let name = node.id.split(' ').next().unwrap();

            for dep in &node.dependencies {
                let dep_name = dep.split(' ').next().unwrap();
                self.graph
                    .add_edge(self.crates[name], self.crates[dep_name], ());
            }
        }
    }

    pub fn display(&self) {
        // Recalculate the list of regressed crates
        let regressed = self
            .graph
            .neighbors_directed(self.root, Direction::Outgoing)
            .map(|node| self.graph.node_weight(node).unwrap().as_str())
            .collect::<Vec<_>>();

        let mut shown = HashSet::new();
        for leaf in self.graph.externals(Direction::Outgoing) {
            self.display_node(leaf, &regressed, &mut shown, 0);
        }
    }

    fn display_node<'a>(
        &'a self,
        node: NodeIndex,
        regressed: &[&'a str],
        shown: &mut HashSet<&'a str>,
        depth: usize,
    ) {
        let mut depth = depth;
        let name = self.graph.node_weight(node).unwrap();

        if !shown.insert(name.as_str()) {
            return;
        }

        if regressed.contains(&name.as_str()) {
            let tab = (0..depth).map(|_| "| ").collect::<String>();
            println!("{}{}", tab, name);

            depth += 1;
        }

        for leaf in self.graph.neighbors_directed(node, Direction::Incoming) {
            self.display_node(leaf, regressed, shown, depth);
        }
    }
}
