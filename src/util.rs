pub fn map2range(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
	(out_min + (out_max - out_min) * (x - in_min) / (in_max - in_min)).clamp(out_min, out_max)
}

macro_rules! unwrap_break {
	($expr:expr) => {
		if let cflow @ ControlFlow::Break(()) = $expr {
			return Ok(cflow);
		}
	};
}
pub(crate) use unwrap_break;
