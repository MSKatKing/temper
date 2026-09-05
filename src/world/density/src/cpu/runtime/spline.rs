use splines::{Interpolation, Key};
use crate::cpu::buffer::BufferId;
use crate::cpu::runtime::{DensityResult, Operation};
use crate::cpu::workspace::{Workspace, WorkspaceStorable};

#[derive(Debug)]
pub enum SplineOrValue<Dst: WorkspaceStorable> {
    Value(f32),
    Spline(BufferId<Dst>)
}

#[derive(Debug)]
pub struct SplinePoint<Dst: WorkspaceStorable> {
    pub location: f32,
    pub derivative: f32,
    pub value: SplineOrValue<Dst>,
}

#[derive(Debug)]
pub struct Spline<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub points: Vec<SplinePoint<Dst>>,
}

impl<Dst: WorkspaceStorable> Operation for Spline<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let mut values = Vec::new();
        for (target_idx, dst) in workspace.get_buffer(self.dst)?.iter().enumerate() {
            let points = self.points
                .iter()
                .map(|point| {
                    Key::new(
                        point.location,
                        match &point.value {
                            SplineOrValue::Value(v) => *v,
                            SplineOrValue::Spline(buf) => workspace.get_buffer(*buf).unwrap()[target_idx],
                        },
                        Interpolation::Bezier(point.derivative),
                    )
                })
                .collect::<Vec<_>>();

            let spline = splines::Spline::from_vec(points);

            values.push(spline.clamped_sample(*dst).unwrap());
        }

        workspace.get_buffer_mut(self.dst)?.copy_from_slice(values.as_slice());

        Ok(())
    }
}
