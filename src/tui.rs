use std::io;

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

    // tui.gmesh_para.vol_phy_list.push(VolPhys {
    //     name: String::from("Rogers"),
    //     phys_id: String::from("101"),
    //     vol_ids: String::from("1,2,3,4,5,56,6,7,6,5,4"),
    // });
    // tui.gmesh_para.vol_phy_list.push(VolPhys {
    //     name: String::from("PEC_T"),
    //     phys_id: String::from("102"),
    //     vol_ids: String::from("1,2,3,4,5,5,5,4"),
    // });
    // tui.gmesh_para.vol_phy_list.push(VolPhys {
    //     name: String::from("Dielec_4"),
    //     phys_id: String::from("104"),
    //     vol_ids: String::from("4,5,56,6,5,4"),
    // });

    // tui.gmesh_para.surf_phy_list.push(SurfPhys {
    //     name: String::from("ABC"),
    //     phys_id: String::from("1"),
    //     surf_ids: String::from("1,2"),
    // });

    // tui.gmesh_para.surf_phy_list.push(SurfPhys {
    //     name: String::from("PEC_S"),
    //     phys_id: String::from("2"),
    //     surf_ids: String::from("3,4"),
    // });

    // tui.gmesh_para.surf_phy_list.push(SurfPhys {
    //     name: String::from("Huy"),
    //     phys_id: String::from("3"),
    //     surf_ids: String::from("4,5,6,7,8"),
    // });
    // tui.gmesh_para.surf_phy_list.push(SurfPhys {
    //     name: String::from("IE"),
    //     phys_id: String::from("9"),
    //     surf_ids: String::from("10,12,13"),
    // });

    ratatui::run(|terminal| tui.run(terminal))
}

enum TypeMode {
    None,
    Volume,
    Surface,
    Mesh,
}

pub struct TUI {
    exit: bool,
    gmesh_para: GmshPara,
    table_state: TableState, //this state is shared

    cur_type: TypeMode,
}

impl TUI {
    fn new() -> Self {
        TUI {
            exit: false,
            gmesh_para: GmshPara::new(),
            table_state: TableState::new(),
            cur_type: TypeMode::None,
        }
    }
}

impl TUI {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            //calling to draw
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;

            //use manipulation process
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_evt) => self.handle_key_event(key_evt)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_evt: crossterm::event::KeyEvent) -> io::Result<()> {
        match (key_evt.kind, key_evt.code) {
            (KeyEventKind::Press, KeyCode::Esc) => self.exit = true,
            (KeyEventKind::Press, KeyCode::Down) => self.table_state_down(),
            (KeyEventKind::Press, KeyCode::Up) => self.table_state_up(),
            (KeyEventKind::Press, KeyCode::Left) => self.type_mode_left(),
            (KeyEventKind::Press, KeyCode::Right) => self.type_mode_right(),
            _ => {}
        }

        Ok(())
    }

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
}

impl Widget for &TUI {
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
        Line::from("Esc: Quit; ↑↓←→ to select; Enter to modify;")
            .centered()
            .yellow()
            .render(bottom_area, buf);

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
