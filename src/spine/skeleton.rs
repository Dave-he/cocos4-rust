use super::animation::SpineAnimation;
use super::bone::SpineBone;

#[derive(Debug, Clone)]
pub struct SpineSlot {
    pub name: String,
    pub bone_name: String,
    pub attachment: Option<String>,
    pub color: [f32; 4],
    pub slot_index: u32,
}

impl SpineSlot {
    pub fn new(name: &str, bone_name: &str) -> Self {
        Self {
            name: name.to_string(),
            bone_name: bone_name.to_string(),
            attachment: None,
            color: [1.0; 4],
            slot_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<SpineBone>,
    pub slots: Vec<SpineSlot>,
    pub animations: Vec<SpineAnimation>,
    pub default_skin: String,
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    current_anim: Option<usize>,
}

impl Skeleton {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bones: Vec::new(),
            slots: Vec::new(),
            animations: Vec::new(),
            default_skin: "default".to_string(),
            x: 0.0, y: 0.0,
            scale_x: 1.0, scale_y: 1.0,
            current_anim: None,
        }
    }

    pub fn add_bone(&mut self, bone: SpineBone) {
        self.bones.push(bone);
    }

    pub fn add_slot(&mut self, slot: SpineSlot) {
        self.slots.push(slot);
    }

    pub fn add_animation(&mut self, anim: SpineAnimation) {
        self.animations.push(anim);
    }

    pub fn play_animation(&mut self, name: &str) {
        if let Some(idx) = self.animations.iter().position(|a| a.name == name) {
            self.animations[idx].play();
            self.current_anim = Some(idx);
        }
    }

    pub fn stop_animation(&mut self) {
        if let Some(idx) = self.current_anim {
            self.animations[idx].stop();
        }
        self.current_anim = None;
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(idx) = self.current_anim {
            self.animations[idx].update(dt);
            for bone in &mut self.bones {
                if let Some(kf) = self.animations[idx].get_interpolated(&bone.name) {
                    bone.x = kf.x;
                    bone.y = kf.y;
                    bone.rotation = kf.rotation;
                    bone.scale_x = kf.scale_x;
                    bone.scale_y = kf.scale_y;
                }
            }
        }
    }

    pub fn get_bone(&self, name: &str) -> Option<&SpineBone> {
        self.bones.iter().find(|b| b.name == name)
    }

    pub fn get_slot(&self, name: &str) -> Option<&SpineSlot> {
        self.slots.iter().find(|s| s.name == name)
    }

    pub fn get_animation(&self, name: &str) -> Option<&SpineAnimation> {
        self.animations.iter().find(|a| a.name == name)
    }

    pub fn get_bone_count(&self) -> usize { self.bones.len() }
    pub fn get_slot_count(&self) -> usize { self.slots.len() }
    pub fn get_animation_count(&self) -> usize { self.animations.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_new() {
        let sk = Skeleton::new("goblin");
        assert_eq!(sk.name, "goblin");
        assert_eq!(sk.get_bone_count(), 0);
    }

    #[test]
    fn test_skeleton_add_bone_slot() {
        let mut sk = Skeleton::new("goblin");
        sk.add_bone(SpineBone::new("root"));
        sk.add_bone(SpineBone::new("head"));
        sk.add_slot(SpineSlot::new("eyes", "head"));
        assert_eq!(sk.get_bone_count(), 2);
        assert_eq!(sk.get_slot_count(), 1);
    }

    #[test]
    fn test_skeleton_animation_play() {
        let mut sk = Skeleton::new("goblin");
        sk.add_bone(SpineBone::new("root"));
        sk.add_animation(SpineAnimation::new("walk", 1.0));
        sk.play_animation("walk");
        sk.update(0.5);
    }

    #[test]
    fn test_skeleton_stop() {
        let mut sk = Skeleton::new("goblin");
        sk.add_animation(SpineAnimation::new("idle", 2.0));
        sk.play_animation("idle");
        sk.stop_animation();
        assert!(sk.current_anim.is_none());
    }

    #[test]
    fn test_skeleton_missing_animation() {
        let mut sk = Skeleton::new("goblin");
        sk.play_animation("nonexistent");
        assert!(sk.current_anim.is_none());
    }
}
