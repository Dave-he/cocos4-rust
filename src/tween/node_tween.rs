use std::sync::{Arc, Mutex};
use crate::math::{Vec3, Quaternion};
use crate::core::scene_graph::NodePtr;
use crate::tween::easing::EasingMethod;
use crate::tween::tween_system::TweenSystem;
use crate::tween::tween::{Tween, tween};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeTweenProp {
    PositionX,
    PositionY,
    PositionZ,
    ScaleX,
    ScaleY,
    ScaleZ,
    ScaleUniform,
}

pub struct NodeTweenBuilder {
    node: NodePtr,
    steps: Vec<NodeTweenStep>,
    easing: EasingMethod,
}

enum NodeTweenStep {
    MoveTo(Vec3, f32),
    MoveBy(Vec3, f32),
    ScaleTo(Vec3, f32),
    ScaleBy(f32, f32),
    FadeTo(f32, f32),
    RotateTo(Quaternion, f32),
    Delay(f32),
    Call(Box<dyn Fn() + Send + Sync>),
}

impl NodeTweenBuilder {
    pub fn new(node: NodePtr) -> Self {
        NodeTweenBuilder {
            node,
            steps: Vec::new(),
            easing: EasingMethod::Linear,
        }
    }

    pub fn with_easing(mut self, easing: EasingMethod) -> Self {
        self.easing = easing;
        self
    }

    pub fn move_to(mut self, target: Vec3, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::MoveTo(target, duration));
        self
    }

    pub fn move_by(mut self, delta: Vec3, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::MoveBy(delta, duration));
        self
    }

    pub fn scale_to(mut self, target: Vec3, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::ScaleTo(target, duration));
        self
    }

    pub fn scale_uniform(mut self, factor: f32, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::ScaleBy(factor, duration));
        self
    }

    pub fn rotate_to(mut self, target: Quaternion, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::RotateTo(target, duration));
        self
    }

    pub fn fade_to(mut self, alpha: f32, duration: f32) -> Self {
        self.steps.push(NodeTweenStep::FadeTo(alpha, duration));
        self
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.steps.push(NodeTweenStep::Delay(seconds));
        self
    }

    pub fn call<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.steps.push(NodeTweenStep::Call(Box::new(f)));
        self
    }

    pub fn build(self) -> Tween {
        let node = self.node;
        let easing = self.easing;
        let mut t = tween();

        for step in self.steps {
            match step {
                NodeTweenStep::MoveTo(target, duration) => {
                    let n = Arc::clone(&node);
                    let from = n.lock().map(|node| node.get_position()).unwrap_or(Vec3::ZERO);
                    let n2 = Arc::clone(&node);
                    let fx = from.x;
                    let fy = from.y;
                    let fz = from.z;
                    let tx = target.x;
                    let ty = target.y;
                    let tz = target.z;
                    let elapsed = Arc::new(Mutex::new(0.0f32));
                    let el = Arc::clone(&elapsed);
                    t = t.call(move || {
                        let mut e = el.lock().unwrap();
                        *e = 0.0;
                    });
                    let elapsed2 = Arc::clone(&elapsed);
                    let n3 = Arc::clone(&n2);
                    t = t.to_single(duration, "__move__", 0.0, 1.0, easing)
                        .call(move || {
                            let progress = {
                                let e = elapsed2.lock().unwrap();
                                (*e / duration).clamp(0.0, 1.0)
                            };
                            let x = fx + (tx - fx) * progress;
                            let y = fy + (ty - fy) * progress;
                            let z = fz + (tz - fz) * progress;
                            if let Ok(mut node) = n3.lock() {
                                node.set_position_xyz(x, y, z);
                            }
                        });
                }
                NodeTweenStep::MoveBy(delta, duration) => {
                    let n = Arc::clone(&node);
                    let from = n.lock().map(|node| node.get_position()).unwrap_or(Vec3::ZERO);
                    let target = Vec3::new(from.x + delta.x, from.y + delta.y, from.z + delta.z);
                    let n2 = Arc::clone(&n);
                    let fx = from.x;
                    let fy = from.y;
                    let fz = from.z;
                    let tx = target.x;
                    let ty = target.y;
                    let tz = target.z;
                    let progress = Arc::new(Mutex::new(0.0f32));
                    let p = Arc::clone(&progress);
                    t = t.to_single(duration, "__moveby__", 0.0, 1.0, easing)
                        .call(move || {
                            let prog = {
                                *p.lock().unwrap()
                            };
                            let x = fx + (tx - fx) * prog;
                            let y = fy + (ty - fy) * prog;
                            let z = fz + (tz - fz) * prog;
                            if let Ok(mut node) = n2.lock() {
                                node.set_position_xyz(x, y, z);
                            }
                        });
                }
                NodeTweenStep::ScaleTo(target, duration) => {
                    let n = Arc::clone(&node);
                    let from = n.lock().map(|node| node.get_scale()).unwrap_or(Vec3::ONE);
                    let n2 = Arc::clone(&n);
                    let fs = from;
                    let ts = target;
                    t = t.to_single(duration, "__scale__", 0.0, 1.0, easing)
                        .call(move || {
                            if let Ok(mut node) = n2.lock() {
                                node.set_scale(ts);
                                let _ = fs;
                            }
                        });
                }
                NodeTweenStep::ScaleBy(factor, duration) => {
                    let n = Arc::clone(&node);
                    let from = n.lock().map(|node| node.get_scale()).unwrap_or(Vec3::ONE);
                    let n2 = Arc::clone(&n);
                    let target = Vec3::new(from.x * factor, from.y * factor, from.z * factor);
                    t = t.to_single(duration, "__scaleby__", 0.0, 1.0, easing)
                        .call(move || {
                            if let Ok(mut node) = n2.lock() {
                                node.set_scale(target);
                            }
                        });
                }
                NodeTweenStep::RotateTo(target, duration) => {
                    let n = Arc::clone(&node);
                    t = t.to_single(duration, "__rotate__", 0.0, 1.0, easing)
                        .call(move || {
                            if let Ok(mut node) = n.lock() {
                                node.set_rotation(target);
                            }
                        });
                }
                NodeTweenStep::FadeTo(_alpha, duration) => {
                    t = t.delay(duration);
                }
                NodeTweenStep::Delay(secs) => {
                    t = t.delay(secs);
                }
                NodeTweenStep::Call(f) => {
                    t = t.call(f);
                }
            }
        }
        t
    }

    pub fn start(self, tween_system: &mut TweenSystem) {
        let t = self.build().start();
        tween_system.add(t);
    }
}

pub fn node_tween(node: NodePtr) -> NodeTweenBuilder {
    NodeTweenBuilder::new(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::core::scene_graph::BaseNode;
    use crate::tween::tween_system::TweenSystem;

    fn make_node(name: &str) -> NodePtr {
        Arc::new(Mutex::new(BaseNode::new(name)))
    }

    #[test]
    fn test_node_tween_builder_new() {
        let node = make_node("A");
        let builder = node_tween(Arc::clone(&node));
        let t = builder.build();
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Idle);
    }

    #[test]
    fn test_move_to_builds_tween() {
        let node = make_node("MoveNode");
        let t = node_tween(Arc::clone(&node))
            .move_to(Vec3::new(10.0, 0.0, 0.0), 1.0)
            .build();
        assert_ne!(t.get_state(), crate::tween::tween::TweenState::Finished);
    }

    #[test]
    fn test_delay_step() {
        let node = make_node("D");
        let mut t = node_tween(Arc::clone(&node))
            .delay(0.5)
            .build()
            .start();
        t.update(0.3);
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Running);
        t.update(0.3);
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Finished);
    }

    #[test]
    fn test_call_step_fires() {
        let node = make_node("C");
        let called = Arc::new(Mutex::new(false));
        let c = Arc::clone(&called);
        let mut t = node_tween(Arc::clone(&node))
            .call(move || { *c.lock().unwrap() = true; })
            .build()
            .start();
        t.update(0.0);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_chain_steps() {
        let node = make_node("Chain");
        let count = Arc::new(Mutex::new(0u32));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);
        let mut t = node_tween(Arc::clone(&node))
            .call(move || { *c1.lock().unwrap() += 1; })
            .delay(0.01)
            .call(move || { *c2.lock().unwrap() += 1; })
            .build()
            .start();
        t.update(1.0);
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn test_start_adds_to_tween_system() {
        let node = make_node("Sys");
        let mut ts = TweenSystem::new();
        node_tween(Arc::clone(&node))
            .delay(10.0)
            .start(&mut ts);
        assert_eq!(ts.count(), 1);
    }

    #[test]
    fn test_with_easing() {
        let node = make_node("E");
        let t = node_tween(Arc::clone(&node))
            .with_easing(EasingMethod::QuadIn)
            .delay(0.1)
            .build();
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Idle);
    }

    #[test]
    fn test_scale_uniform_step() {
        let node = make_node("S");
        let t = node_tween(Arc::clone(&node))
            .scale_uniform(2.0, 0.5)
            .build();
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Idle);
    }

    #[test]
    fn test_rotate_to_step() {
        let node = make_node("R");
        let rot = Quaternion::IDENTITY;
        let t = node_tween(Arc::clone(&node))
            .rotate_to(rot, 0.5)
            .build();
        assert_eq!(t.get_state(), crate::tween::tween::TweenState::Idle);
    }
}
