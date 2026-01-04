#![allow(dead_code)]

///All use String because this will be put in .geo script as String
#[derive(Debug, Clone)]
pub struct VolPhys {
    pub name: String,
    pub phys_id: String,
    pub vol_ids: String,
}

#[derive(Debug, Clone)]
pub struct SurfPhys {
    pub name: String,
    pub phys_id: String,
    pub surf_ids: String,
}

#[derive(Debug, Clone)]
pub struct MeshPara {
    pub max_size: String, // if None, Mesh Max Size will not be set
                          //add more parameters in future
}

#[derive(Debug, Clone)]
pub struct GmshPara {
    pub geometry_file: String, //filename of the geometry, eg. xxx.step
    pub vol_phy_list: Vec<VolPhys>,
    pub surf_phy_list: Vec<SurfPhys>,
    pub mesh_paras: MeshPara,
}

impl GmshPara {
    pub fn new() -> Self {
        GmshPara {
            geometry_file: String::new(),
            vol_phy_list: Vec::new(),
            surf_phy_list: Vec::new(),
            mesh_paras: MeshPara {
                max_size: String::new(),
            },
        }
    }

    pub fn apply_mesh(&self) {
        todo!()
    }
}
