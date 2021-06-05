use crate::elementwise::elementwise_single::{BinaryElementwise, BinaryFunc, TernaryElementwise, TernaryFunc};
use alumina_core::{
	base_ops::OpSpecification,
	errors::{GradientError, OpBuildError},
	grad::GradientContext,
	graph::{merge_graphs, Node, NodeID},
};

/// Calculates the elementwise minimum (min) of input1 and input2.
///
/// The output node has the same shape as the input.
pub fn min<I1, I2>(input1: I1, input2: I2) -> Result<Node, OpBuildError>
where
	I1: Into<Node>,
	I2: Into<Node>,
{
	let input1 = input1.into();
	let input2 = input2.into();
	merge_graphs(&[input1.graph(), input2.graph()]);
	let output = input1
		.graph()
		.new_node(input1.shape())
		.set_name_unique(&format!("min({},{})", input1, input2));
	let _op = Min::new_default(input1, input2, output.clone())
		.build()
		.expect("Error building Min Op");
	Ok(output)
}

pub type Min = BinaryElementwise<MinFunc>;

#[derive(Clone, Debug, Default)]
pub struct MinFunc {}

impl BinaryFunc for MinFunc {
	#[inline]
	fn calc(&self, input1: f32, input2: f32) -> f32 {
		input1.min(input2)
	}

	fn type_name(&self) -> &'static str {
		"Min"
	}

	fn grad(
		&self,
		ctx: &mut GradientContext,
		input1: &NodeID,
		input2: &NodeID,
		output: &NodeID,
	) -> Result<(), GradientError> {
		// TODO combine into single backward
		let _op = MinBack::new_default(
			ctx.node(input1),
			ctx.node(input2),
			ctx.grad_of(output),
			ctx.grad_of(input1),
		)
		.build()?;
		let _op = MinBack::new_default(
			ctx.node(input2),
			ctx.node(input1),
			ctx.grad_of(output),
			ctx.grad_of(input2),
		)
		.build()?;
		Ok(())
	}
}

pub type MinBack = TernaryElementwise<MinBackFunc>;

/// input1 = an input of min
/// input2 = an input of min
/// input3 = grad of output of min
/// returns grad for input1
#[derive(Clone, Debug, Default)]
pub struct MinBackFunc {}
impl TernaryFunc for MinBackFunc {
	#[inline]
	fn calc(&self, input1: f32, input2: f32, input3: f32) -> f32 {
		if input1 < input2 {
			input3
		} else {
			0.0
		}
	}

	fn type_name(&self) -> &'static str {
		"MinBack"
	}

	fn grad(
		&self,
		_ctx: &mut GradientContext,
		_input1: &NodeID,
		_input2: &NodeID,
		_input3: &NodeID,
		_output: &NodeID,
	) -> Result<(), GradientError> {
		Err(GradientError::Unimplemented)
	}
}

#[cfg(test)]
mod tests {
	use super::min;
	use alumina_core::{graph::Node, init::uniform};
	use alumina_test::{grad_numeric_test::GradNumericTest, relatively_close::RelClose};

	use indexmap::indexset;
	use ndarray::arr0;

	#[test]
	fn forward_test() {
		let input1 = Node::new(&[13, 33]).set_name("input1");
		let input2 = Node::new(&[13, 33]).set_name("input2");

		let output = min(&input1, &input2).unwrap();

		input1.set_value(arr0(1.25));
		input2.set_value(arr0(1.25));
		assert!(output
			.calc()
			.unwrap()
			.all_relatively_close(&arr0(1.25), ::std::f32::EPSILON));

		input1.set_value(arr0(1.25));
		input2.set_value(arr0(-0.8));
		assert!(output
			.calc()
			.unwrap()
			.all_relatively_close(&arr0(-0.8), ::std::f32::EPSILON));

		input1.set_value(arr0(-0.8));
		input2.set_value(arr0(1.25));
		assert!(output
			.calc()
			.unwrap()
			.all_relatively_close(&arr0(-0.8), ::std::f32::EPSILON));

		input1.set_value(arr0(-0.8));
		input2.set_value(arr0(-0.8));
		assert!(output
			.calc()
			.unwrap()
			.all_relatively_close(&arr0(-0.8), ::std::f32::EPSILON));
	}

	#[test]
	fn grad_numeric_test() {
		let input1 = Node::new(&[13, 33]).set_name("input1").set_init(uniform(-1.0, 1.0));
		let input2 = Node::new(&[13, 33]).set_name("input2").set_init(uniform(-1.0, 1.0));

		let output = min(&input1, &input2).unwrap();

		GradNumericTest::new(&output, &indexset![&input1, &input2])
			.step_size(1e-3)
			.tolerance(4e-3)
			.run();
	}

	#[test]
	fn grad_numeric_low_tol_test() {
		let input1 = Node::new(&[13, 33]).set_name("input1").set_init(uniform(0.1, 1.0));
		let input2 = Node::new(&[13, 33]).set_name("input2").set_init(uniform(-1.0, -0.1));

		let output = min(&input1, &input2).unwrap();

		GradNumericTest::new(&output, &indexset![&input1, &input2])
			.expect_zero(&input1, ::std::f32::EPSILON)
			.run();
	}

	// #[test]
	// fn grad_numeric_shared_input_test() {
	// 	let input1 = Node::new(&[13, 33])
	// 		.set_name("input1")
	// 		.set_init(Initialiser::uniform(-1.0, 1.0));

	// 	let output = min(&input1, &input1);

	// 	GradNumericTest::new(&output, &indexset![&input1])
	// 		.step_size(1e-3)
	// 		.tolerance(4e-3)
	// 		.run();
	// }
}
