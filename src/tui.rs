use std::fs;
use std::io;
use std::process::Child;

use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use ratatui::DefaultTerminal;
use ratatui::layout::Spacing;
use ratatui::prelude::*;

use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::*;

use crate::gmsh_ctl::*;

pub fn tui_start(geometry_filename: String) -> io::Result<()> {
    let mut tui = TUI::new();

    //assigning geometry file name
    tui.gmesh_para.geometry_file = geometry_filename;

    //start a Gmsh Child Process first to visualize the geometry
    let (gmsh_handle_result, temp_file_name) = tui.gmesh_para.apply_mesh();

    if let Ok(gmsh_handle) = gmsh_handle_result {
        tui.gmsh_handle = Some(gmsh_handle);
    }

    let tui_res = ratatui::run(|terminal| tui.run(terminal));

    //kill Gmsh Child Process
    if let Some(mut gmsh_child) = tui.gmsh_handle.take() {
        if let Err(e) = gmsh_child.kill() {
            panic!("Failed to kill Gmsh: {}", e)
        }
    }
    // clean up temporary file
    if let Err(e) = fs::remove_file(temp_file_name) {
        panic!("Failed to clean up: {}", e)
    }

    tui_res
}

enum TypeMode {
    None,
    Volume,
    Surface,
    Mesh,
}

enum OperaMode {
    Select,
    Modify,
}

enum ModifyType {
    VolName,
    VolPID,
    VolVID,
    SurName,
    SurPID,
    SurSID,
    MeshVal,
    None,
}
struct Cursor {
    begin_x: u16,
    begin_y: u16,
    char_idx: u16,
    modify_type: ModifyType,
} //the actual x=begin_x + char_idx, y=begin_y

pub struct TUI {
    exit: bool,
    gmesh_para: GmshPara,
    table_state: TableState, //this state is shared
    cur_type: TypeMode,

    input_buf: Vec<String>,
    opreation_mode: OperaMode,

    cursor: Cursor,
    gmsh_handle: Option<Child>,
}

impl TUI {
    fn new() -> Self {
        TUI {
            exit: false,
            gmesh_para: GmshPara::new(),
            table_state: TableState::new(),
            cur_type: TypeMode::None,
            input_buf: Vec::new(),
            opreation_mode: OperaMode::Select,
            cursor: Cursor {
                begin_x: 0,
                begin_y: 0,
                char_idx: 0,
                modify_type: ModifyType::None,
            },
            gmsh_handle: None,
        }
    }
}

impl TUI {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            //calling to draw
            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area());

                //plot cursor in modifying mode
                if let OperaMode::Modify = self.opreation_mode {
                    frame.set_cursor_position(Position::new(
                        self.cursor.begin_x + self.cursor.char_idx,
                        self.cursor.begin_y,
                    ));
                }
            })?;

            //use manipulation process
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_evt) => self.handle_key_event(key_evt)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_evt: crossterm::event::KeyEvent) -> io::Result<()> {
        match self.opreation_mode {
            OperaMode::Select => match (key_evt.kind, key_evt.code) {
                (KeyEventKind::Press, KeyCode::Esc) => self.exit = true,
                (KeyEventKind::Press, KeyCode::Down) => self.table_state_down(),
                (KeyEventKind::Press, KeyCode::Up) => self.table_state_up(),
                (KeyEventKind::Press, KeyCode::Left) => self.type_mode_left(),
                (KeyEventKind::Press, KeyCode::Right) => self.type_mode_right(),

                (KeyEventKind::Press, KeyCode::Enter) => self.select_to_modify(),
                (KeyEventKind::Press, KeyCode::Char('a')) => {
                    if let Some(mut child) = self.gmsh_handle.take() {
                        //kill the previous Gmsh Child Process first
                        if let Err(e) = child.kill() {
                            panic!("{}", e)
                        }
                    }
                    if let Ok(gmsh_handle) = self.gmesh_para.apply_mesh().0 {
                        self.gmsh_handle = Some(gmsh_handle);
                    }
                }

                (KeyEventKind::Press, KeyCode::Backspace | KeyCode::Delete) => {
                    self.delete_selected()
                }
                _ => {}
            },
            OperaMode::Modify => match (key_evt.kind, key_evt.code) {
                (KeyEventKind::Press, KeyCode::Esc) => {
                    self.opreation_mode = OperaMode::Select;
                    self.cursor.modify_type = ModifyType::None;
                }
                (KeyEventKind::Press, KeyCode::Enter) => self.confirm_modification(),

                (KeyEventKind::Press, KeyCode::Tab) => self.modify_tab(),
                (KeyEventKind::Press, KeyCode::Char(ch)) => self.char_insert(ch),
                (KeyEventKind::Press, KeyCode::Backspace) => self.char_backspace(), //delete the char before cursor
                (KeyEventKind::Press, KeyCode::Delete) => self.char_delete(), //delete the char selected by the cursor
                (KeyEventKind::Press, KeyCode::Left) => self.cursor_left(),
                (KeyEventKind::Press, KeyCode::Right) => self.cursor_right(),

                _ => {}
            },
        }

        Ok(())
    }

    ///////////////////////////////////// Select Mode
    fn table_state_down(&mut self) {
        if let Some(idx) = self.table_state.selected() {
            match self.cur_type {
                TypeMode::Volume => {
                    if idx < self.gmesh_para.vol_phy_list.len() {
                        self.table_state.select(Some(idx + 1));
                    } else {
                        self.table_state.select(None);
                    }
                }
                TypeMode::Surface => {
                    if idx < self.gmesh_para.surf_phy_list.len() {
                        self.table_state.select(Some(idx + 1));
                    } else {
                        self.table_state.select(None);
                    }
                }
                TypeMode::Mesh => {
                    //if more parameters exist, this need change
                    if idx == 0 {
                        self.table_state.select(Some(0));
                    } else {
                        self.table_state.select(None);
                    }
                }
                _ => {}
            }
        } else {
            match self.cur_type {
                TypeMode::Volume => {
                    self.table_state.select(Some(0));
                }
                TypeMode::Surface => {
                    self.table_state.select(Some(0));
                }
                TypeMode::Mesh => {
                    self.table_state.select(Some(0));
                }
                TypeMode::None => {
                    self.cur_type = TypeMode::Volume;
                    self.table_state.select(Some(0));
                }
            }
        }
    }

    fn table_state_up(&mut self) {
        if let Some(idx) = self.table_state.selected() {
            match self.cur_type {
                TypeMode::Volume => {
                    if idx > 0 {
                        self.table_state.select(Some(idx - 1));
                    } else {
                        self.table_state.select(None);
                    }
                }
                TypeMode::Surface => {
                    if idx > 0 {
                        self.table_state.select(Some(idx - 1));
                    } else {
                        self.table_state.select(None);
                    }
                }
                TypeMode::Mesh => {
                    if idx > 0 {
                        self.table_state.select(Some(idx - 1));
                    } else {
                        self.table_state.select(None);
                    }
                }
                _ => {}
            }
        } else {
            match self.cur_type {
                TypeMode::Volume => {
                    self.table_state
                        .select(Some(self.gmesh_para.vol_phy_list.len()));
                }
                TypeMode::Surface => {
                    self.table_state
                        .select(Some(self.gmesh_para.surf_phy_list.len()));
                }
                TypeMode::Mesh => {
                    //this need change when more parameters exist
                    self.table_state.select(Some(0));
                }
                TypeMode::None => {
                    self.cur_type = TypeMode::Volume;
                    self.table_state
                        .select(Some(self.gmesh_para.vol_phy_list.len()));
                }
            }
        }
    }

    fn type_mode_left(&mut self) {
        match self.cur_type {
            TypeMode::Volume => {
                self.cur_type = TypeMode::None;
                self.table_state.select(None);
            }
            TypeMode::Surface => {
                self.cur_type = TypeMode::Volume;
                if let Some(idx) = self.table_state.selected()
                    && idx > self.gmesh_para.vol_phy_list.len()
                {
                    self.table_state
                        .select(Some(self.gmesh_para.vol_phy_list.len()));
                }
                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                }
            }
            TypeMode::Mesh => {
                self.cur_type = TypeMode::Surface;
                if let Some(idx) = self.table_state.selected()
                    && idx > self.gmesh_para.surf_phy_list.len()
                {
                    self.table_state
                        .select(Some(self.gmesh_para.surf_phy_list.len()));
                }
                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                }
            }
            TypeMode::None => {
                self.cur_type = TypeMode::Mesh;
                self.table_state.select(Some(0));
            }
        }
    }

    fn type_mode_right(&mut self) {
        match self.cur_type {
            TypeMode::Volume => {
                self.cur_type = TypeMode::Surface;
                if let Some(idx) = self.table_state.selected()
                    && idx > self.gmesh_para.surf_phy_list.len()
                {
                    self.table_state
                        .select(Some(self.gmesh_para.surf_phy_list.len()));
                }
                if self.table_state.selected().is_none() {
                    self.table_state.select(Some(0));
                }
            }
            TypeMode::Surface => {
                self.cur_type = TypeMode::Mesh;
                self.table_state.select(Some(0));
            }
            TypeMode::Mesh => {
                self.cur_type = TypeMode::None;
                self.table_state.select(None);
            }
            TypeMode::None => {
                self.cur_type = TypeMode::Volume;
                self.table_state.select(Some(0));
            }
        }
    }

    fn select_to_modify(&mut self) {
        if let Some(idx) = self.table_state.selected() {
            self.opreation_mode = OperaMode::Modify;
            //setting up input buffer
            match self.cur_type {
                TypeMode::Volume => {
                    self.input_buf.clear();
                    if idx < self.gmesh_para.vol_phy_list.len() {
                        //exsiting parameters selected
                        let temp_vol_phy = self.gmesh_para.vol_phy_list[idx].clone();

                        self.input_buf.push(temp_vol_phy.name);
                        self.input_buf.push(temp_vol_phy.phys_id);
                        self.input_buf.push(temp_vol_phy.vol_ids);

                        //set cursor
                        self.cursor.modify_type = ModifyType::VolName;
                    } else {
                        //adding parameters
                        self.input_buf.push(String::new());
                        self.input_buf.push(String::new());
                        self.input_buf.push(String::new());

                        //set cursor
                        self.cursor.modify_type = ModifyType::VolName;
                    }

                    self.cursor.char_idx = self.input_buf[0].len() as u16;
                }

                TypeMode::Surface => {
                    self.input_buf.clear();
                    if idx < self.gmesh_para.surf_phy_list.len() {
                        //exsiting parameters selected
                        let temp_surf_phy = self.gmesh_para.surf_phy_list[idx].clone();

                        self.input_buf.push(temp_surf_phy.name);
                        self.input_buf.push(temp_surf_phy.phys_id);
                        self.input_buf.push(temp_surf_phy.surf_ids);

                        //set cursor
                        self.cursor.modify_type = ModifyType::SurName;
                    } else {
                        //adding parameters
                        self.input_buf.push(String::new());
                        self.input_buf.push(String::new());
                        self.input_buf.push(String::new());

                        //set cursor
                        self.cursor.modify_type = ModifyType::SurName;
                    }
                    self.cursor.char_idx = self.input_buf[0].len() as u16;
                }
                TypeMode::Mesh => {
                    self.input_buf.clear();
                    match idx {
                        0 => {
                            self.input_buf.push(String::from("MaxSize"));
                            self.input_buf
                                .push(self.gmesh_para.mesh_paras.max_size.clone());

                            //set cursor
                            self.cursor.modify_type = ModifyType::MeshVal;
                            self.cursor.char_idx = self.input_buf[1].len() as u16;
                        }
                        _ => {
                            self.input_buf.push(String::new());
                            self.input_buf.push(String::new());

                            //set cursor
                            self.cursor.modify_type = ModifyType::MeshVal;
                            self.cursor.char_idx = 0;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(select_idx) = self.table_state.selected() {
            match self.cur_type {
                TypeMode::Volume => {
                    if select_idx < self.gmesh_para.vol_phy_list.len() {
                        self.gmesh_para.vol_phy_list.remove(select_idx);
                    }
                }
                TypeMode::Surface => {
                    if select_idx < self.gmesh_para.surf_phy_list.len() {
                        self.gmesh_para.surf_phy_list.remove(select_idx);
                    }
                }
                _ => {}
            }
        }
    }

    /////////////////////////// Modify Mode
    fn modify_tab(&mut self) {
        match self.cursor.modify_type {
            ModifyType::VolName => {
                self.cursor.modify_type = ModifyType::VolPID;
                self.cursor.char_idx = self.input_buf[1].len() as u16;
            }
            ModifyType::VolPID => {
                self.cursor.modify_type = ModifyType::VolVID;
                self.cursor.char_idx = self.input_buf[2].len() as u16;
            }
            ModifyType::VolVID => {
                self.cursor.modify_type = ModifyType::VolName;
                self.cursor.char_idx = self.input_buf[0].len() as u16;
            }
            ModifyType::SurName => {
                self.cursor.modify_type = ModifyType::SurPID;
                self.cursor.char_idx = self.input_buf[1].len() as u16;
            }
            ModifyType::SurPID => {
                self.cursor.modify_type = ModifyType::SurSID;
                self.cursor.char_idx = self.input_buf[2].len() as u16;
            }
            ModifyType::SurSID => {
                self.cursor.modify_type = ModifyType::SurName;
                self.cursor.char_idx = self.input_buf[0].len() as u16;
            }

            _ => {}
        }
    }

    fn char_insert(&mut self, ch: char) {
        match self.cursor.modify_type {
            ModifyType::VolName | ModifyType::SurName => {
                let char_idx = self.cursor.char_idx;
                self.input_buf[0].insert(char_idx as usize, ch);
                self.cursor.char_idx += 1;
            }
            ModifyType::VolPID | ModifyType::SurPID => {
                if ch.is_ascii_digit() {
                    let char_idx = self.cursor.char_idx;
                    self.input_buf[1].insert(char_idx as usize, ch);
                    self.cursor.char_idx += 1;
                }
            }
            ModifyType::VolVID | ModifyType::SurSID => {
                if ch.is_ascii_digit() || ch == ',' {
                    let char_idx = self.cursor.char_idx;
                    self.input_buf[2].insert(char_idx as usize, ch);
                    self.cursor.char_idx += 1;
                }
            }

            ModifyType::MeshVal => {
                if ch.is_ascii_digit() || ch == '.' {
                    let char_idx = self.cursor.char_idx;
                    self.input_buf[1].insert(char_idx as usize, ch);
                    self.cursor.char_idx += 1;
                }
            }
            _ => {}
        }
    }

    fn char_backspace(&mut self) {
        match self.cursor.modify_type {
            ModifyType::VolName | ModifyType::SurName => {
                let char_idx = self.cursor.char_idx;
                if char_idx > 0 {
                    self.input_buf[0].remove((char_idx - 1) as usize);
                    self.cursor.char_idx -= 1;
                }
            }
            ModifyType::VolPID | ModifyType::SurPID => {
                let char_idx = self.cursor.char_idx;
                if char_idx > 0 {
                    self.input_buf[1].remove((char_idx - 1) as usize);
                    self.cursor.char_idx -= 1;
                }
            }
            ModifyType::VolVID | ModifyType::SurSID => {
                let char_idx = self.cursor.char_idx;
                if char_idx > 0 {
                    self.input_buf[2].remove((char_idx - 1) as usize);
                    self.cursor.char_idx -= 1;
                }
            }

            ModifyType::MeshVal => {
                let char_idx = self.cursor.char_idx;
                if char_idx > 0 {
                    self.input_buf[1].remove((char_idx - 1) as usize);
                    self.cursor.char_idx -= 1;
                }
            }
            _ => {}
        }
    }

    fn char_delete(&mut self) {
        match self.cursor.modify_type {
            ModifyType::VolName | ModifyType::SurName => {
                let char_idx = self.cursor.char_idx;
                if char_idx < self.input_buf[0].len() as u16 && (!self.input_buf[0].is_empty()) {
                    self.input_buf[0].remove(char_idx as usize);
                }
            }
            ModifyType::VolPID | ModifyType::SurPID => {
                let char_idx = self.cursor.char_idx;
                if char_idx < self.input_buf[1].len() as u16 && (!self.input_buf[1].is_empty()) {
                    self.input_buf[1].remove(char_idx as usize);
                }
            }
            ModifyType::VolVID | ModifyType::SurSID => {
                let char_idx = self.cursor.char_idx;
                if char_idx < self.input_buf[2].len() as u16 && (!self.input_buf[2].is_empty()) {
                    self.input_buf[2].remove(char_idx as usize);
                }
            }

            ModifyType::MeshVal => {
                let char_idx = self.cursor.char_idx;
                if char_idx < self.input_buf[1].len() as u16 && (!self.input_buf[1].is_empty()) {
                    self.input_buf[1].remove(char_idx as usize);
                }
            }
            _ => {}
        }
    }

    fn cursor_left(&mut self) {
        match self.cursor.modify_type {
            ModifyType::VolName => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolVID;
                    self.cursor.char_idx = self.input_buf[2].len() as u16;
                }
            }
            ModifyType::VolPID => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolName;
                    self.cursor.char_idx = self.input_buf[0].len() as u16;
                }
            }
            ModifyType::VolVID => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolPID;
                    self.cursor.char_idx = self.input_buf[1].len() as u16;
                }
            }
            ModifyType::SurName => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurSID;
                    self.cursor.char_idx = self.input_buf[2].len() as u16;
                }
            }
            ModifyType::SurPID => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurName;
                    self.cursor.char_idx = self.input_buf[0].len() as u16;
                }
            }
            ModifyType::SurSID => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurPID;
                    self.cursor.char_idx = self.input_buf[1].len() as u16;
                }
            }
            ModifyType::MeshVal => {
                if self.cursor.char_idx > 0 {
                    self.cursor.char_idx -= 1;
                }
            }
            _ => {}
        }
    }

    fn cursor_right(&mut self) {
        match self.cursor.modify_type {
            ModifyType::VolName => {
                if self.cursor.char_idx < self.input_buf[0].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolPID;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::VolPID => {
                if self.cursor.char_idx < self.input_buf[1].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolVID;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::VolVID => {
                if self.cursor.char_idx < self.input_buf[2].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::VolName;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::SurName => {
                if self.cursor.char_idx < self.input_buf[0].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurPID;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::SurPID => {
                if self.cursor.char_idx < self.input_buf[1].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurSID;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::SurSID => {
                if self.cursor.char_idx < self.input_buf[2].len() as u16 {
                    self.cursor.char_idx += 1;
                } else {
                    self.cursor.modify_type = ModifyType::SurName;
                    self.cursor.char_idx = 0;
                }
            }
            ModifyType::MeshVal => {
                if self.cursor.char_idx < self.input_buf[1].len() as u16 {
                    self.cursor.char_idx += 1;
                }
            }

            _ => {}
        }
    }

    fn confirm_modification(&mut self) {
        self.opreation_mode = OperaMode::Select;
        self.cursor.modify_type = ModifyType::None;

        if let Some(selected_idx) = self.table_state.selected() {
            match self.cur_type {
                TypeMode::Volume => {
                    if selected_idx < self.gmesh_para.vol_phy_list.len() {
                        //changing existing parameters
                        self.gmesh_para.vol_phy_list[selected_idx] = VolPhys {
                            name: self.input_buf[0].clone(),
                            phys_id: self.input_buf[1].clone(),
                            vol_ids: self.input_buf[2].clone(),
                        };
                    } else {
                        //new parameters
                        self.gmesh_para.vol_phy_list.push(VolPhys {
                            name: self.input_buf[0].clone(),
                            phys_id: self.input_buf[1].clone(),
                            vol_ids: self.input_buf[2].clone(),
                        });
                    }
                }
                TypeMode::Surface => {
                    if selected_idx < self.gmesh_para.surf_phy_list.len() {
                        //changing existing parameters
                        self.gmesh_para.surf_phy_list[selected_idx] = SurfPhys {
                            name: self.input_buf[0].clone(),
                            phys_id: self.input_buf[1].clone(),
                            surf_ids: self.input_buf[2].clone(),
                        };
                    } else {
                        //new parameters
                        self.gmesh_para.surf_phy_list.push(SurfPhys {
                            name: self.input_buf[0].clone(),
                            phys_id: self.input_buf[1].clone(),
                            surf_ids: self.input_buf[2].clone(),
                        });
                    }
                }
                TypeMode::Mesh => match selected_idx {
                    0 => {
                        self.gmesh_para.mesh_paras.max_size = self.input_buf[1].clone();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

impl Widget for &mut TUI {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [up_area, bottom_area] =
            Layout::vertical(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                .areas(area);

        let [vol_area, surf_area, mesh_area] = Layout::horizontal(vec![
            Constraint::Percentage(37),
            Constraint::Percentage(37),
            Constraint::Percentage(26),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(up_area);

        //instructions

        let [bottom_left, bottom_right] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)])
                .areas(bottom_area);

        Line::from(String::from("Geometry File: ") + &self.gmesh_para.geometry_file)
            .blue()
            .render(bottom_left, buf);

        match self.opreation_mode {
            OperaMode::Select => {
                Line::from("| Esc: quit | ↑↓←→: select | Enter: modify | a: apply to Gmsh |")
                    .yellow()
                    .render(bottom_right, buf);
            }
            OperaMode::Modify => {
                Line::from("| Esc: quit | Enter: confirm | Tab: change selection |")
                    .yellow()
                    .render(bottom_right, buf);
            }
        }

        //main UI rendering

        let vol_block = Block::new()
            .title("Physical Volume")
            .borders(Borders::ALL)
            .merge_borders(MergeStrategy::Exact);

        let surf_block = Block::new()
            .title("Physical Surface")
            .borders(Borders::ALL)
            .merge_borders(MergeStrategy::Exact);

        let mesh_block = Block::new()
            .title("Mesh Parameters")
            .borders(Borders::ALL)
            .merge_borders(MergeStrategy::Exact);

        //Physical Volume Table
        let mut vol_rows = Vec::new();
        row_convertion_vol(&self.gmesh_para.vol_phy_list, &mut vol_rows);

        let vol_table = Table::new(
            vol_rows,
            vec![
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Fill(1),
            ],
        )
        .block(vol_block)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

        //Physical Surface Table
        let mut surf_rows = Vec::new();
        row_convertion_surf(&self.gmesh_para.surf_phy_list, &mut surf_rows);

        let surf_table = Table::new(
            surf_rows,
            vec![
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Fill(1),
            ],
        )
        .block(surf_block)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

        //Mesh Parameters Table
        let mut mesh_rows = Vec::new();
        row_convertion_mesh(&self.gmesh_para.mesh_paras, &mut mesh_rows);

        let mesh_table = Table::new(
            mesh_rows,
            vec![Constraint::Percentage(30), Constraint::Fill(1)],
        )
        .block(mesh_block)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

        let mut vol_state = TableState::new();
        let mut surf_state = TableState::new();
        let mut mesh_state = TableState::new();
        match self.cur_type {
            TypeMode::Volume => vol_state = self.table_state.clone(),
            TypeMode::Surface => surf_state = self.table_state.clone(),
            TypeMode::Mesh => mesh_state = self.table_state.clone(),
            _ => {}
        }

        StatefulWidget::render(vol_table, vol_area, buf, &mut vol_state);
        StatefulWidget::render(surf_table, surf_area, buf, &mut surf_state);
        StatefulWidget::render(mesh_table, mesh_area, buf, &mut mesh_state);

        //render popup dialog in OpreaMode::Modify
        if let OperaMode::Modify = self.opreation_mode {
            let popup_area = popup_area(area, 60, 3);
            Widget::render(Clear, popup_area, buf); //clean the background for popup

            match self.cur_type {
                TypeMode::Volume => {
                    let [name_area, pid_area, vid_area] = Layout::horizontal([
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Fill(1),
                    ])
                    .spacing(Spacing::Overlap(1))
                    .areas(popup_area);

                    let name_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Name");
                    let pid_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Physical ID");
                    let vid_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Volume ID");

                    Paragraph::new(self.input_buf[0].clone())
                        .block(name_block)
                        .render(name_area, buf);
                    Paragraph::new(self.input_buf[1].clone())
                        .block(pid_block)
                        .render(pid_area, buf);
                    Paragraph::new(self.input_buf[2].clone())
                        .block(vid_block)
                        .render(vid_area, buf);

                    //set cursor
                    match self.cursor.modify_type {
                        ModifyType::VolName => {
                            self.cursor.begin_x = name_area.x + 1;
                            self.cursor.begin_y = name_area.y + 1;
                        }
                        ModifyType::VolPID => {
                            self.cursor.begin_x = pid_area.x + 1;
                            self.cursor.begin_y = pid_area.y + 1;
                        }
                        ModifyType::VolVID => {
                            self.cursor.begin_x = vid_area.x + 1;
                            self.cursor.begin_y = vid_area.y + 1;
                        }
                        _ => {}
                    }
                }
                TypeMode::Surface => {
                    let [name_area, pid_area, sid_area] = Layout::horizontal([
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Fill(1),
                    ])
                    .spacing(Spacing::Overlap(1))
                    .areas(popup_area);

                    let name_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Name");
                    let pid_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Physical ID");
                    let sid_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Surface ID");

                    Paragraph::new(self.input_buf[0].clone())
                        .block(name_block)
                        .render(name_area, buf);
                    Paragraph::new(self.input_buf[1].clone())
                        .block(pid_block)
                        .render(pid_area, buf);
                    Paragraph::new(self.input_buf[2].clone())
                        .block(sid_block)
                        .render(sid_area, buf);

                    //set cursor
                    match self.cursor.modify_type {
                        ModifyType::SurName => {
                            self.cursor.begin_x = name_area.x + 1;
                            self.cursor.begin_y = name_area.y + 1;
                        }
                        ModifyType::SurPID => {
                            self.cursor.begin_x = pid_area.x + 1;
                            self.cursor.begin_y = pid_area.y + 1;
                        }
                        ModifyType::SurSID => {
                            self.cursor.begin_x = sid_area.x + 1;
                            self.cursor.begin_y = sid_area.y + 1;
                        }
                        _ => {}
                    }
                }
                TypeMode::Mesh => {
                    let [name_area, val_area] =
                        Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)])
                            .spacing(Spacing::Overlap(1))
                            .areas(popup_area);

                    let name_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Name");
                    let val_block = Block::bordered()
                        .merge_borders(MergeStrategy::Exact)
                        .title("Value");

                    Paragraph::new(self.input_buf[0].clone())
                        .block(name_block)
                        .render(name_area, buf);

                    Paragraph::new(self.input_buf[1].clone())
                        .block(val_block)
                        .render(val_area, buf);

                    //set cursor
                    match self.cursor.modify_type {
                        ModifyType::MeshVal => {
                            self.cursor.begin_x = val_area.x + 1;
                            self.cursor.begin_y = val_area.y + 1;
                        }
                        _ => {}
                    }
                }
                TypeMode::None => {}
            }
        }
    }
}

fn row_convertion_vol(vol_list: &Vec<VolPhys>, rows: &mut Vec<Row>) {
    rows.clear();

    for vol_phy in vol_list.clone() {
        let row = Row::new(vec![vol_phy.name, vol_phy.phys_id, vol_phy.vol_ids]);
        rows.push(row);
    }
    rows.push(Row::new(["Enter", "To", "Add"]));
}

fn row_convertion_surf(surf_list: &Vec<SurfPhys>, rows: &mut Vec<Row>) {
    rows.clear();

    for surf_phy in surf_list.clone() {
        let row = Row::new(vec![surf_phy.name, surf_phy.phys_id, surf_phy.surf_ids]);
        rows.push(row);
    }

    rows.push(Row::new(["Enter", "To", "Add"]));
}

fn row_convertion_mesh(mesh_para: &MeshPara, rows: &mut Vec<Row>) {
    rows.clear();

    let tmp = mesh_para.clone();

    rows.push(Row::new(vec![String::from("MaxSize"), tmp.max_size]));
}

fn popup_area(area: Rect, perc_x: u16, length_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(length_y)]).flex(layout::Flex::Center);
    let horizontal =
        Layout::horizontal([Constraint::Percentage(perc_x)]).flex(layout::Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
