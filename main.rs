use std::process::{Command, Stdio};
use std::fs::File;
use std::io::Write;
use std::env;

fn run_command(cmd: &str) {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
    };

    match status {
        Ok(s) if s.success() => {}
        _ => println!("Ошибка при выполнении команды: {}", cmd),
    }
}

fn clone_repo(repo_url: &str, dest_folder: &str) {
    println!("Клонирование репозитория {} в папку {}...", repo_url, dest_folder);
    // Git сам создаёт папки, кавычки не нужны
    let cmd = format!("git clone {} {}", repo_url, dest_folder);
    run_command(&cmd);
}

fn create_run_bat() {
    let mut f = match File::create("run.bat") {
        Ok(file) => file,
        Err(_) => { println!("Не удалось создать run.bat"); return; }
    };
    let content = r#"@echo off
echo *** NStor ComfyUI cuda129 runner ***
uv run ComfyUI\main.py --windows-standalone-build
pause
"#;
    f.write_all(content.as_bytes()).unwrap();
    println!("Файл run.bat создан");
}

fn main() {
    #[allow(unsafe_code)]
    unsafe {
        // UTF-8 консоль
        run_command("chcp 65001 >nul");

        // Переменная окружения UV_LINK_MODE
        env::set_var("UV_LINK_MODE", "copy");
    }

    // Проверка Git
    let git_ok = Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !git_ok {
        println!("Требуется установленный Git!");
        println!("Скачайте его с https://git-scm.com/download/win");
        println!("После этого повторите установку.");
        run_command("pause");
        return;
    }

    println!("*** NStor ComfyUI cuda129 installer ***\n");

    println!("Проверка UV...");
    let uv_installed = Command::new("uv")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if uv_installed {
        println!("UV найден. Обновляем...");
        run_command("uv self update");
    } else {
        println!("UV не найден. Устанавливаем...");
        run_command("powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\"");
    }

    println!("\nКлонирование репозиториев ComfyUI и ComfyUI-Manager через Git...");
    clone_repo("https://github.com/comfyanonymous/ComfyUI.git", "ComfyUI");
    clone_repo("https://github.com/ltdrdata/ComfyUI-Manager.git", "ComfyUI\\custom_nodes\\ComfyUI-Manager");

    println!("\nСоздание venv на Python 3.12...");
    run_command("uv venv --python 3.12 --seed");

    println!("\nУстановка Torch с CUDA 12.9...");
    run_command("uv pip install torch --extra-index-url https://download.pytorch.org/whl/cu129");

    println!("\nУстановка зависимостей ComfyUI...");
    run_command("uv pip install -r ComfyUI/requirements.txt");

    println!("\nУстановка зависимостей ComfyUI-Manager...");
    run_command("uv pip install -r ComfyUI\\custom_nodes\\ComfyUI-Manager\\requirements.txt");

    create_run_bat();

    println!("\nЗапуск ComfyUI...");
    run_command("uv run ComfyUI/main.py --windows-standalone-build");
}
