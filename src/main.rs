#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use std::time::Duration;
use eframe::egui;
use std::os::windows::process::CommandExt;
use std::process::{Command, Child};
use std::fs;
use std::env;
use std::path::PathBuf;

// --- Ресурсы (оставляем те же) ---
const WINWS_EXE: &[u8] = include_bytes!("../bin/winws.exe");
const CYGWIN_DLL: &[u8] = include_bytes!("../bin/cygwin1.dll");
const DIVERT_DLL: &[u8] = include_bytes!("../bin/WinDivert.dll");
const DIVERT_SYS: &[u8] = include_bytes!("../bin/WinDivert64.sys");
const BIN_QUIC: &[u8] = include_bytes!("../bin/quic_initial_www_google_com.bin");
const BIN_GOOGLE: &[u8] = include_bytes!("../bin/tls_clienthello_www_google_com.bin");
const LIST_ALL: &[u8] = include_bytes!("../lists/ipset-all.txt");
const LIST_EXCLUDE_IP: &[u8] = include_bytes!("../lists/ipset-exclude.txt");
const LIST_EXCLUDE_HOST: &[u8] = include_bytes!("../lists/list-exclude.txt");
const LIST_GENERAL: &[u8] = include_bytes!("../lists/list-general.txt");
const LIST_GOOGLE: &[u8] = include_bytes!("../lists/list-google.txt");

struct ZapretApp {
    child: Option<Child>,
    status_msg: String,
}

impl Default for ZapretApp {
    fn default() -> Self {
        Self { 
            child: None,
            status_msg: "Система готова".to_string(),
        }
    }
}

impl ZapretApp {
    fn get_core_path() -> PathBuf {
        // Папка будет: C:\Users\Имя\AppData\Local\Temp\zapret_service_data
        env::temp_dir().join("zapret_service_data")
    }

    fn unpack_files() -> std::io::Result<()> {
        let core_path = Self::get_core_path();
        if !core_path.exists() { fs::create_dir_all(&core_path)?; }
        fs::write(core_path.join("winws.exe"), WINWS_EXE)?;
        fs::write(core_path.join("cygwin1.dll"), CYGWIN_DLL)?;
        fs::write(core_path.join("WinDivert.dll"), DIVERT_DLL)?;
        fs::write(core_path.join("WinDivert64.sys"), DIVERT_SYS)?;
        fs::write(core_path.join("quic_initial_www_google_com.bin"), BIN_QUIC)?;
        fs::write(core_path.join("tls_clienthello_www_google_com.bin"), BIN_GOOGLE)?;
        fs::write(core_path.join("ipset-all.txt"), LIST_ALL)?;
        fs::write(core_path.join("ipset-exclude.txt"), LIST_EXCLUDE_IP)?;
        fs::write(core_path.join("list-exclude.txt"), LIST_EXCLUDE_HOST)?;
        fs::write(core_path.join("list-general.txt"), LIST_GENERAL)?;
        fs::write(core_path.join("list-google.txt"), LIST_GOOGLE)?;
        Ok(())
    }

    fn kill_all_winws() {
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "winws.exe", "/T"])
            .creation_flags(0x08000000)
            .status();
    }
}

impl eframe::App for ZapretApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Устанавливаем темную тему
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("Zapret GUI").strong().size(24.0));
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(15.0);

                if self.child.is_none() {
                    ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::LIGHT_GRAY));
                    ui.add_space(15.0);

                    // Стилизованная кнопка запуска
                    let start_btn = egui::Button::new(egui::RichText::new("Запустить"))
                        .fill(egui::Color32::from_rgb(34, 139, 34))
                        .min_size(egui::vec2(220.0, 45.0));

                if ui.add(start_btn).clicked() {
                    self.status_msg = "Запуск...".to_string();
                    
                    // 1. Убиваем старые процессы
                    Self::kill_all_winws();
                    
                    // 2. Ждем 300мс, чтобы ОС освободила файлы
                    thread::sleep(Duration::from_millis(300));

                    // 3. Распаковываем и запускаем
                    if let Err(e) = Self::unpack_files() {
                        self.status_msg = format!("Ошибка доступа: {}", e);
                    } else {
                            let core_path = Self::get_core_path();
                            let args = [
                                "--wf-tcp=80,443,2053,2083,2087,2096,8443", "--wf-udp=443,19294-19344,50000-50100",
                                "--filter-udp=443", "--hostlist=list-general.txt", "--hostlist-exclude=list-exclude.txt", "--ipset-exclude=ipset-exclude.txt", "--dpi-desync=fake", "--dpi-desync-repeats=6", "--dpi-desync-fake-quic=quic_initial_www_google_com.bin", 
                                "--new",
                                "--filter-udp=19294-19344,50000-50100", "--filter-l7=discord,stun", "--dpi-desync=fake", "--dpi-desync-repeats=6", 
                                "--new",
                                "--filter-tcp=2053,2083,2087,2096,8443", "--hostlist-domains=discord.media", "--dpi-desync=fake,fakedsplit", "--dpi-desync-repeats=6", "--dpi-desync-fooling=ts", "--dpi-desync-fakedsplit-pattern=0x00", "--dpi-desync-fake-tls=tls_clienthello_www_google_com.bin", 
                                "--new",
                                "--filter-tcp=443", "--hostlist=list-google.txt", "--ip-id=zero", "--dpi-desync=fake,fakedsplit", "--dpi-desync-repeats=6", "--dpi-desync-fooling=ts", "--dpi-desync-fakedsplit-pattern=0x00", "--dpi-desync-fake-tls=tls_clienthello_www_google_com.bin", 
                                "--new",
                                "--filter-tcp=80,443", "--hostlist=list-general.txt", "--hostlist-exclude=list-exclude.txt", "--ipset-exclude=ipset-exclude.txt", "--dpi-desync=fake,fakedsplit", "--dpi-desync-repeats=6", "--dpi-desync-fooling=ts", "--dpi-desync-fakedsplit-pattern=0x00", "--dpi-desync-fake-tls=tls_clienthello_www_google_com.bin",
                                "--new",
                                "--filter-udp=443", "--ipset=ipset-all.txt", "--hostlist-exclude=list-exclude.txt", "--ipset-exclude=ipset-exclude.txt", "--dpi-desync=fake", "--dpi-desync-repeats=6", "--dpi-desync-fake-quic=quic_initial_www_google_com.bin",
                                "--new",
                                "--filter-tcp=80,443", "--ipset=ipset-all.txt", "--hostlist-exclude=list-exclude.txt", "--ipset-exclude=ipset-exclude.txt", "--dpi-desync=fake,fakedsplit", "--dpi-desync-repeats=6", "--dpi-desync-fooling=ts", "--dpi-desync-fakedsplit-pattern=0x00", "--dpi-desync-fake-tls=tls_clienthello_www_google_com.bin",
                                "--new",
                                "--ipset=ipset-all.txt", "--ipset-exclude=ipset-exclude.txt", "--dpi-desync=fake", "--dpi-desync-autottl=2", "--dpi-desync-repeats=12", "--dpi-desync-any-protocol=1", "--dpi-desync-fake-unknown-udp=quic_initial_www_google_com.bin", "--dpi-desync-cutoff=n3"
                            ];

                            let c = Command::new(core_path.join("winws.exe"))
                                .current_dir(&core_path)
                                .args(&args)
                                .creation_flags(0x08000000)
                                .spawn();

                            if let Ok(child_proc) = c {
                                self.child = Some(child_proc);
                            }
                        }
                    }
                } else {
                    ui.add_space(5.0);
                    ui.colored_label(egui::Color32::LIGHT_GREEN, "✔ Служба запущенна");
                    ui.label(egui::RichText::new("YouTube и Discord работают").small());
                    ui.add_space(20.0);
                    
                    // Стилизованная кнопка остановки
                    let stop_btn = egui::Button::new(egui::RichText::new("❌ ОСТАНОВИТЬ"))
                        .fill(egui::Color32::from_rgb(178, 34, 34))
                        .min_size(egui::vec2(220.0, 45.0));

                    if ui.add(stop_btn).clicked() {
                        Self::kill_all_winws();
                        self.child = None;
                        self.status_msg = "Служба остановлена".to_string();
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([350.0, 220.0]) // Чуть увеличили размер окна
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Zapret Ultimate GUI",
        options,
        Box::new(|cc| {
            // Глобальная настройка скруглений
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.widgets.inactive.rounding = egui::Rounding::same(10.0);
            style.visuals.widgets.hovered.rounding = egui::Rounding::same(10.0);
            style.visuals.widgets.active.rounding = egui::Rounding::same(10.0);
            cc.egui_ctx.set_style(style);
            
            Ok(Box::new(ZapretApp::default()))
        }),
    )
}