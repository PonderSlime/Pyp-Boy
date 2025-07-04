#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
	Stat,
    Inv,
    Data,
	Map,
	Radio,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Stat => 0,
            MenuItem::Inv => 1,
			MenuItem::Data => 2,
            MenuItem::Map => 3,
			MenuItem::Radio => 4,
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub enum StatSubMenu {
    General,
    Status,
    Settings,
}

impl From<StatSubMenu> for usize {
    fn from(input: StatSubMenu) -> usize {
        match input {
            StatSubMenu::General => 0,
            StatSubMenu::Status => 1,
            StatSubMenu::Settings => 2,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InvSubMenu {
    Weapons,
    Apparel,
    Aid,
	Misc,
	Junk,
	Mods,
	Ammo,
}

impl From<InvSubMenu> for usize {
    fn from(input: InvSubMenu) -> usize {
        match input {
            InvSubMenu::Weapons => 0,
            InvSubMenu::Apparel => 1,
            InvSubMenu::Aid => 2,
			InvSubMenu::Misc => 3,
            InvSubMenu::Junk => 4,
            InvSubMenu::Mods => 5,
			InvSubMenu::Ammo => 6,
      
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DataSubMenu {
    Quests,
    Workshops,
    Stats,
}

impl From<DataSubMenu> for usize {
    fn from(input: DataSubMenu) -> usize {
        match input {
            DataSubMenu::Quests => 0,
            DataSubMenu::Workshops => 1,
            DataSubMenu::Stats => 2,
        }
    }
}