import {
    MenubarContent,
    MenubarItem, MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger
} from "@components/ui/menubar.tsx";
import {invoke} from "@tauri-apps/api/core";
import {CopyPlus, Globe} from "lucide-react";
import {useTranslation} from "react-i18next";

export default function File() {
    const { t } = useTranslation('home');

    async function addMod() {
        await invoke("add_mod");
    }

    async function openNexus() {
        await invoke('open_search_browser');
    }

    async function test() {
        await invoke('fix_mod_folder');
    }

    async function close() {
        await invoke('close');
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>{t("file")}</MenubarTrigger>
            <MenubarContent>
                <MenubarItem onClick={addMod}>
                    <CopyPlus size={14} className="mr-2" />
                    {t("addMod")}
                </MenubarItem>
                <MenubarItem onClick={openNexus}>
                    <Globe size={14} className="mr-2" />
                    {t("openNexusmod")}
                </MenubarItem>
                <MenubarSeparator />
                <MenubarItem onClick={close}>
                    {t("close")}
                </MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}