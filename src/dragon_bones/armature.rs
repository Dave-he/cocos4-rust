use super::animation::Animation;
use super::bone::Bone;
use super::slot::Slot;

#[derive(Debug, Clone)]
pub struct Armature {
    pub name: String,
    pub version: String,
    pub frame_rate: f32,
    pub bones: Vec<Bone>,
    pub slots: Vec<Slot>,
    pub animations: Vec<Animation>,
    current_animation: Option<String>,
}

impl Armature {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "5.5".to_string(),
            frame_rate: 30.0,
            bones: Vec::new(),
            slots: Vec::new(),
            animations: Vec::new(),
            current_animation: None,
        }
    }

    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
    }

    pub fn add_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
    }

    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    pub fn get_bone(&self, name: &str) -> Option<&Bone> {
        self.bones.iter().find(|b| b.name == name)
    }

    pub fn get_bone_mut(&mut self, name: &str) -> Option<&mut Bone> {
        self.bones.iter_mut().find(|b| b.name == name)
    }

    pub fn get_slot(&self, name: &str) -> Option<&Slot> {
        self.slots.iter().find(|s| s.name == name)
    }

    pub fn get_animation(&self, name: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.name == name)
    }

    pub fn get_animation_mut(&mut self, name: &str) -> Option<&mut Animation> {
        self.animations.iter_mut().find(|a| a.name == name)
    }

    pub fn play_animation(&mut self, name: &str) {
        if let Some(anim) = self.animations.iter_mut().find(|a| a.name == name) {
            anim.play();
            self.current_animation = Some(name.to_string());
        }
    }

    pub fn stop_animation(&mut self) {
        if let Some(name) = &self.current_animation {
            if let Some(anim) = self.animations.iter_mut().find(|a| &a.name == name) {
                anim.stop();
            }
        }
        self.current_animation = None;
    }

    pub fn get_current_animation(&self) -> Option<&Animation> {
        if let Some(name) = &self.current_animation {
            self.get_animation(name)
        } else {
            None
        }
    }

    pub fn advance_time(&mut self, dt: f32) {
        if let Some(name) = self.current_animation.clone() {
            if let Some(anim) = self.animations.iter_mut().find(|a| a.name == name) {
                anim.advance_time(dt);
            }
        }
    }

    pub fn update_world_transform(&mut self) {
        for i in 0..self.bones.len() {
            if i == 0 {
                let b = &mut self.bones[i];
                b.world_position = b.position;
                b.world_rotation = b.rotation;
                b.world_scale = b.scale;
            } else {
                let pw = self.bones[i - 1].world_position;
                let pr = self.bones[i - 1].world_rotation;
                let ps = self.bones[i - 1].world_scale;
                self.bones[i].update_world_transform(&pw, pr, &ps);
            }
        }
    }

    pub fn get_bone_count(&self) -> usize {
        self.bones.len()
    }
    pub fn get_slot_count(&self) -> usize {
        self.slots.len()
    }
    pub fn get_animation_count(&self) -> usize {
        self.animations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dragon_bones::animation::AnimationPlayMode;

    #[test]
    fn test_armature_new() {
        let a = Armature::new("hero");
        assert_eq!(a.name, "hero");
        assert_eq!(a.get_bone_count(), 0);
    }

    #[test]
    fn test_armature_add_bone_slot() {
        let mut a = Armature::new("hero");
        a.add_bone(Bone::new("root"));
        a.add_bone(Bone::new("torso"));
        a.add_slot(Slot::new("body", "torso"));
        assert_eq!(a.get_bone_count(), 2);
        assert_eq!(a.get_slot_count(), 1);
    }

    #[test]
    fn test_armature_animation() {
        let mut a = Armature::new("hero");
        let mut anim = Animation::new("walk", 1.0);
        anim.play_mode = AnimationPlayMode::Loop;
        a.add_animation(anim);
        a.play_animation("walk");
        assert!(a.get_current_animation().is_some());
        a.advance_time(0.5);
        let anim = a.get_current_animation().unwrap();
        assert!((anim.get_current_time() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_armature_world_transform() {
        let mut a = Armature::new("hero");
        let mut root = Bone::new("root");
        root.position = [0.0, 0.0];
        let mut child = Bone::new("child");
        child.position = [10.0, 0.0];
        a.add_bone(root);
        a.add_bone(child);
        a.update_world_transform();
        let child = a.get_bone("child").unwrap();
        assert!((child.world_position[0] - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_armature_stop_animation() {
        let mut a = Armature::new("hero");
        a.add_animation(Animation::new("idle", 2.0));
        a.play_animation("idle");
        assert!(a.get_current_animation().is_some());
        a.stop_animation();
        assert!(a.get_current_animation().is_none());
    }
}
