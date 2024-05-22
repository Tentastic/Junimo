import {MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarTrigger} from "@components/ui/menubar.tsx";
import {Settings} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import {useTranslation} from "react-i18next";


export default function Tools() {
    const { t } = useTranslation('home');

    async function openConfig() {
        await invoke("open_config")
    }

    async function openExport() {
        await invoke("open_export")
    }

    async function openImport() {
        await invoke("open_import")
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>{t("tools")}</MenubarTrigger>
            <MenubarContent>
                <MenubarItem onClick={openImport}>{t("import")}</MenubarItem>
                <MenubarItem onClick={openExport}>{t("export")}</MenubarItem>
                <MenubarSeparator />
                <MenubarItem onClick={openConfig}>
                    <Settings size={16} className="mr-1" /> {t("settings")}
                </MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}