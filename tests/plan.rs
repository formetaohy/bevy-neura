use bevy_neura::{ModelPlan, Roles};
use neura::{Element, Graph, Shape};

#[test]
fn a_plan_names_every_host_extent_it_binds() {
    let _ = ModelPlan::build(|graph| {
        let batch = graph.free(4);
        let observation = graph.input(Shape::matrix(4, 2).freed(&[(2, batch)]), Element::Single);
        Roles::new()
            .axis("batch", batch)
            .input("observation", observation)
    });
}

#[test]
fn a_plan_names_its_outputs_as_roles() {
    let _ = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let prediction = graph.mul(observation, graph.fill(Shape::scalar(), 2.0));
        Roles::new()
            .input("observation", observation)
            .output("prediction", prediction)
    });
}

#[test]
#[should_panic(expected = "a model names the axes")]
fn a_plan_that_names_no_extent_of_its_graph_panics() {
    let _ = ModelPlan::build(|graph| {
        let batch = graph.free(4);
        let observation = graph.input(Shape::matrix(4, 2).freed(&[(2, batch)]), Element::Single);
        Roles::new().input("observation", observation)
    });
}

#[test]
#[should_panic(expected = "a model names the axes")]
fn a_plan_that_binds_a_device_authored_extent_panics() {
    let _ = ModelPlan::build(|graph| {
        let lengths = graph.input(Shape::vector(2), Element::Single);
        let ragged = graph.ragged(8, lengths);
        Roles::new().axis("rows", ragged.extent)
    });
}

#[test]
#[should_panic(expected = "two host extents answer to the role batch")]
fn two_extents_answer_to_one_role_once() {
    let _ = ModelPlan::build(|graph| {
        let batch = graph.free(4);
        let rows = graph.free(2);
        Roles::new().axis("batch", batch).axis("batch", rows)
    });
}

#[test]
#[should_panic(expected = "two inputs answer to the role observation")]
fn two_inputs_answer_to_one_role_once() {
    let _ = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(4, 2), Element::Single);
        Roles::new()
            .input("observation", observation)
            .input("observation", observation)
    });
}

#[test]
#[should_panic(expected = "two outputs answer to the role prediction")]
fn two_outputs_answer_to_one_role_once() {
    let _ = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(4, 2), Element::Single);
        Roles::new()
            .output("prediction", observation)
            .output("prediction", observation)
    });
}

#[test]
#[should_panic(expected = "names one tensor of a model")]
fn a_role_that_names_two_tensors_reaches_no_plan() {
    let _ = ModelPlan::build(|graph| {
        let observation = graph.input(Shape::matrix(2, 2), Element::Single);
        let scaled = graph.mul(observation, graph.fill(Shape::scalar(), 2.0));
        Roles::new()
            .input("tensor", observation)
            .output("tensor", scaled)
    });
}

#[test]
#[should_panic(expected = "a tensor of another graph reached this graph")]
fn a_value_of_another_graph_reaches_no_plan() {
    let _ = ModelPlan::build(|_graph| {
        let foreign = Graph::new();
        let observation = foreign.input(Shape::matrix(4, 2), Element::Single);
        Roles::new().input("observation", observation)
    });
}
