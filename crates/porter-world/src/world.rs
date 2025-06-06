use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::path::Path;

use porter_cast::CastFile;
use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;

use porter_math::Axis;

use crate::Instance;

/// A 3d world definition.
#[derive(Debug, Clone)]
pub struct World {
    /// A collection of instances for this world.
    pub instances: Vec<Instance>,
    /// The up axis for this world.
    pub up_axis: Axis,
}

impl World {
    /// Constructs a new instance of world.
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            up_axis: Axis::Z,
        }
    }

    /// Scales the world by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for instance in &mut self.instances {
            instance.scale(factor);
        }
    }

    /// Saves the world to the given file path using the cast format.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut root = CastNode::root();

        let meta_node = root.create(CastId::Metadata);

        meta_node
            .create_property(CastPropertyId::String, "a")
            .push("DTZxPorter");

        meta_node
            .create_property(CastPropertyId::String, "s")
            .push("Exported by PorterLib");

        let up_axis = match self.up_axis {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        };

        meta_node
            .create_property(CastPropertyId::String, "up")
            .push(up_axis);

        for instance in &self.instances {
            instance.save(&mut root);
        }

        let writer = BufWriter::new(File::create(path.as_ref().with_extension("cast"))?);

        let mut file = CastFile::new();

        file.push(root);
        file.write(writer)?;

        Ok(())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
