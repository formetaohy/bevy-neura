use crate::model::Model;
use crate::roles::Roles;
use neura::{Graph, Runtime};

pub struct ModelPlan {
    graph: Graph<'static>,
    roles: Roles,
}

impl ModelPlan {
    pub fn build(roles_of: impl FnOnce(&Graph<'static>) -> Roles) -> Self {
        let graph: Graph<'static> = Graph::new();
        let roles = roles_of(&graph);
        roles.assert_binds(&graph);
        Self { graph, roles }
    }

    pub fn attach(self, runtime: &Runtime) -> Model {
        let (graph, roles) = self.retained();
        let weights = runtime.weights(&graph);
        let program = runtime.compile(&graph, &weights);
        Model::of(roles, program)
    }

    pub fn attach_tuned(self, runtime: &Runtime) -> Model {
        let (graph, roles) = self.retained();
        let weights = runtime.weights(&graph);
        let program = runtime.tune(&graph, &weights);
        Model::of(roles, program)
    }

    pub fn share(self, runtime: &Runtime, source: &Model) -> Model {
        let (graph, roles) = self.retained();
        let weights = source.weights().clone();
        runtime.rebind(&weights, &graph);
        let program = runtime.compile(&graph, &weights);
        Model::of(roles, program)
    }

    fn retained(self) -> (Graph<'static>, Roles) {
        for value in self.roles.retains() {
            self.graph.retain(value);
        }
        (self.graph, self.roles)
    }
}
