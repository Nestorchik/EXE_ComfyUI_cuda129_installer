#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

#define MAX_CMD 1024

void run_command(const char *cmd) {
    int ret = system(cmd);
    if (ret != 0) {
        printf("Ошибка при выполнении команды: %s\n", cmd);
    }
}

void clone_repo(const char *repo_url, const char *dest_folder) {
    char cmd[MAX_CMD];
    snprintf(cmd, sizeof(cmd), "git clone \"%s\" \"%s\"", repo_url, dest_folder);
    printf("Клонирование репозитория %s в папку %s...\n", repo_url, dest_folder);
    run_command(cmd);
}

void create_run_bat() {
    FILE *f = fopen("run.bat", "w");
    if (!f) { printf("Не удалось создать run.bat\n"); return; }
    fprintf(f,
        "@echo off\n"
        "chcp 65001 >nul\n"
        "echo *** NStor ComfyUI cuda129 runner ***\n"
        "echo Запуск ComfyUI...\n"
        "uv run ComfyUI\\main.py --windows-standalone-build\n"
        "pause\n"
    );
    fclose(f);
    printf("Файл run.bat создан\n");
}

int main(void) {
    // UTF-8 консоль
    SetConsoleOutputCP(CP_UTF8);
    SetConsoleCP(CP_UTF8);
    system("chcp 65001 >nul");

    // Переменная окружения UV_LINK_MODE=copy
    _putenv("UV_LINK_MODE=copy");

    // Проверка Git
    if (system("git --version >nul 2>nul") != 0) {
        printf("Требуется установленный Git!\n");
        printf("Скачайте его с https://git-scm.com/download/win\n");
        printf("После этого повторите установку.\n");
        system("pause");
        return 1;
    }

    // Баннер в начале
    printf("*** NStor ComfyUI cuda129 installer ***\n\n");

    printf("Проверка UV...\n");
    int uv_installed = system("uv --version >nul 2>nul") == 0;
    if (uv_installed) {
        printf("UV найден. Обновляем...\n");
        run_command("uv self update");
    } else {
        printf("UV не найден. Устанавливаем...\n");
        run_command("powershell -ExecutionPolicy ByPass -c \"irm https://astral.sh/uv/install.ps1 | iex\"");
    }

    printf("\nКлонирование репозиториев ComfyUI и ComfyUI-Manager через Git...\n");
    clone_repo("https://github.com/comfyanonymous/ComfyUI.git", "ComfyUI");
    clone_repo("https://github.com/ltdrdata/ComfyUI-Manager.git", "ComfyUI\\custom_nodes\\ComfyUI-Manager");

    printf("\nСоздание venv на Python 3.12...\n");
    run_command("uv venv --python 3.12 --seed");

    printf("\nУстановка Torch с CUDA 12.9...\n");
    run_command("uv pip install torch --extra-index-url https://download.pytorch.org/whl/cu129");

    printf("\nУстановка зависимостей ComfyUI...\n");
    run_command("uv pip install -r ComfyUI/requirements.txt");

    printf("\nУстановка зависимостей ComfyUI-Manager...\n");
    run_command("uv pip install -r ComfyUI\\custom_nodes\\ComfyUI-Manager\\requirements.txt");

    create_run_bat();

    printf("\nЗапуск ComfyUI...\n");
    run_command("uv run ComfyUI/main.py --windows-standalone-build");

    return 0;
}
