#![allow(dead_code)]

use std::{
    fs::{self, read_to_string},
    io,
    path::Path,
    process::{Child, Stdio},
};

use serde::{Deserialize, Serialize};

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
        let mut new_gmsh_para = GmshPara {
            geometry_file: String::new(),
            vol_phy_list: Vec::new(),
            surf_phy_list: Vec::new(),
            mesh_paras: MeshPara {
                max_size: String::new(),
            },
        };

        //check if there is a termsh_cache.json
        let cache_filename = "termsh_cache.json";

        if Path::new(cache_filename).exists() {
            let cache_content = read_to_string(cache_filename).expect("Error Reading Cache File");
            let phys_pair_list: PhysPairList =
                serde_json::from_str(&cache_content).expect("Error Deserialize");

            for vol_pair in phys_pair_list.vol_pairs {
                new_gmsh_para.vol_phy_list.push(VolPhys {
                    name: vol_pair.name,
                    phys_id: vol_pair.phys_id,
                    vol_ids: String::new(),
                });
            }

            for sur_pair in phys_pair_list.sur_pairs {
                new_gmsh_para.surf_phy_list.push(SurfPhys {
                    name: sur_pair.name,
                    phys_id: sur_pair.phys_id,
                    surf_ids: String::new(),
                });
            }
        }

        new_gmsh_para
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

    pub fn save_cache(&self) {
        //Save PhysPairList to termsh_cache.json when existing the program
        let mut phys_pair_list = PhysPairList {
            vol_pairs: Vec::new(),
            sur_pairs: Vec::new(),
        };

        for vol_phys in self.vol_phy_list.clone() {
            phys_pair_list.vol_pairs.push(PhysPair {
                name: vol_phys.name,
                phys_id: vol_phys.phys_id,
            });
        }

        for sur_phys in self.surf_phy_list.clone() {
            phys_pair_list.sur_pairs.push(PhysPair {
                name: sur_phys.name,
                phys_id: sur_phys.phys_id,
            });
        }

        let json_str = serde_json::to_string_pretty(&phys_pair_list).expect("Error Serialize");
        if let Err(e) = fs::write("termsh_cache.json", json_str) {
            panic!("Error writing to cache: {}", e)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PhysPair {
    name: String,
    phys_id: String,
}

//use serde to serial/deserial to termsh_cache.json
#[derive(Serialize, Deserialize)]
struct PhysPairList {
    vol_pairs: Vec<PhysPair>,
    sur_pairs: Vec<PhysPair>,
}
