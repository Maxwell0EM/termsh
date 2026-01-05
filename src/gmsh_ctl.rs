#![allow(dead_code)]

use std::{
    fs, io,
    path::Path,
    process::{Child, Stdio},
};

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

    pub fn apply_mesh(&self) -> (io::Result<Child>, String) {
        let mut scirpt_content = String::new();

        //generate geometry file readin scripts
        //using this is better than Merge
        scirpt_content += &format!(
            "SetFactory(\"OpenCASCADE\");\n\
            v()=ShapeFromFile(\"{}\");\n",
            self.geometry_file
        );

        //generate geometry healing scripts
        scirpt_content += &format!(
            "/* Heal the step file */\n\
            BooleanFragments{{ Volume{{:}}; Surface {{:}}; Delete; }}{{}}\n\
            Coherence;\n"
        );

        //generate Physical Volume scripts
        scirpt_content += &format!("/* Physical Volume Grouping */\n");

        for vol_phys in &self.vol_phy_list {
            if vol_phys.vol_ids.is_empty() {
                continue; //skip empty content
            }
            scirpt_content += &format!(
                "Physical Volume(\"{}\",{})={{{}}};\n",
                vol_phys.name, vol_phys.phys_id, vol_phys.vol_ids
            );
        }

        //generate Physical Surface scripts
        scirpt_content += &format!("/* Physical Surface Grouping */\n");

        for sur_phys in &self.surf_phy_list {
            if sur_phys.surf_ids.is_empty() {
                continue; //skip empty content
            }
            scirpt_content += &format!(
                "Physical Surface(\"{}\",{})={{{}}};\n",
                sur_phys.name, sur_phys.phys_id, sur_phys.surf_ids
            );
        }

        //generate Mesh Parameter scripts
        scirpt_content += &format!("/* Mesh Setting */\n");

        if !self.mesh_paras.max_size.is_empty() {
            scirpt_content += &format!("Mesh.MeshSizeMax={};\n", self.mesh_paras.max_size);
        }

        //perform meshing scripts
        scirpt_content += &format!("Mesh 3;\n");

        //get filename_prefix
        let filename_prefix = Path::new(&self.geometry_file)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        //export content to temporary script file
        let temp_script_file_name = filename_prefix.to_owned() + "_temp.geo";
        if let Err(err) = fs::write(&temp_script_file_name, scirpt_content) {
            panic!("Error when writing to temporary script file: {}", err)
        }

        //spawn Gmsh Child Process
        let gmsh_child_handle = std::process::Command::new("gmsh")
            .arg(&temp_script_file_name)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to start gmsh");

        (Ok(gmsh_child_handle), temp_script_file_name)
    }

    pub fn apply_mesh_and_save_to_nas(&self) -> (io::Result<Child>, String) {
        let mut scirpt_content = String::new();

        //generate geometry file readin scripts
        //using this is better than Merge
        scirpt_content += &format!(
            "SetFactory(\"OpenCASCADE\");\n\
            v()=ShapeFromFile(\"{}\");\n",
            self.geometry_file
        );

        //generate geometry healing scripts
        scirpt_content += &format!(
            "/* Heal the step file */\n\
            BooleanFragments{{ Volume{{:}}; Surface {{:}}; Delete; }}{{}}\n\
            Coherence;\n"
        );

        //generate Physical Volume scripts
        scirpt_content += &format!("/* Physical Volume Grouping */\n");

        for vol_phys in &self.vol_phy_list {
            if vol_phys.vol_ids.is_empty() {
                continue; //skip empty content
            }
            scirpt_content += &format!(
                "Physical Volume(\"{}\",{})={{{}}};\n",
                vol_phys.name, vol_phys.phys_id, vol_phys.vol_ids
            );
        }

        //generate Physical Surface scripts
        scirpt_content += &format!("/* Physical Surface Grouping */\n");

        for sur_phys in &self.surf_phy_list {
            if sur_phys.surf_ids.is_empty() {
                continue; //skip empty content
            }
            scirpt_content += &format!(
                "Physical Surface(\"{}\",{})={{{}}};\n",
                sur_phys.name, sur_phys.phys_id, sur_phys.surf_ids
            );
        }

        //generate Mesh Parameter scripts
        scirpt_content += &format!("/* Mesh Setting */\n");

        if !self.mesh_paras.max_size.is_empty() {
            scirpt_content += &format!("Mesh.MeshSizeMax={};\n", self.mesh_paras.max_size);
        }

        //perform meshing scripts
        scirpt_content += &format!("Mesh 3;\n");

        //get filename_prefix
        let filename_prefix = Path::new(&self.geometry_file)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        //generate save to .nas file scripts
        //Mesh.Format=31; Save the mesh in bdf format
        //Mesh.BdfFieldFormat=1; Save in small format
        //Mesh.SaveAll=0; Only save Physical Objects
        //Mesh.SaveElementTagType=2; Save the tag using Physical IDs
        scirpt_content += &format!(
            "Mesh.Format=31;\n\
            Mesh.BdfFieldFormat=1;\n\
            Mesh.SaveAll=0;\n\
            Mesh.SaveElementTagType=2;\n\
            Save \"{}\";",
            filename_prefix.to_owned() + ".nas"
        );

        //export content to temporary script file
        let temp_script_file_name = filename_prefix.to_owned() + "_temp.geo";
        if let Err(err) = fs::write(&temp_script_file_name, scirpt_content) {
            panic!("Error when writing to temporary script file: {}", err)
        }

        //spawn Gmsh Child Process
        let gmsh_child_handle = std::process::Command::new("gmsh")
            .arg(&temp_script_file_name)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to start gmsh");

        (Ok(gmsh_child_handle), temp_script_file_name)
    }
}
