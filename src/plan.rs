use crate::model::Model;
use crate::roles::Roles;
use neura::{Graph, Runtime};

pub struct ModelPlan {
    graph: Graph,
    roles: Roles,
}

impl ModelPlan {
    pub fn build(roles_of: impl FnOnce(&Graph) -> Roles) -> Self {
        let graph: Graph = Graph::new();
        let roles = roles_of(&graph);
        roles.assert_binds(&graph);
        Self { graph, roles }
    }

    pub fn attach(self, runtime: &Runtime) -> Model {
        let (graph, roles) = self.retained();
        let weights = runtime.weights(&graph);
        Model::of(roles, runtime.compile(&graph, &weights))
    }

    pub fn tune(self, runtime: &Runtime) -> Model {
        let (graph, roles) = self.retained();
        let weights = runtime.weights(&graph);
        Model::of(roles, runtime.tune(&graph, &weights))
    }

    pub fn share(self, runtime: &Runtime, source: &Model) -> Model {
        let (graph, roles) = self.retained();
        Model::of(roles, runtime.compile(&graph, source.weights()))
    }

    fn retained(self) -> (Graph, Roles) {
        for value in self.roles.retains() {
            self.graph.retain(value);
        }
        (self.graph, self.roles)
    }
}
