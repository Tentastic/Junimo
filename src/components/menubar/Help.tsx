import {
    MenubarContent,
    MenubarItem, MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger
} from "@components/ui/menubar.tsx";
import {useTranslation} from "react-i18next";
import {invoke} from "@tauri-apps/api/core";


export default function Help() {
    const { t } = useTranslation('home');

    async function openSmapi() {
        await invoke("open_smapi");
    }

    async function openUpdater() {
        await invoke("open_updater");
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>{t("help")}</MenubarTrigger>
            <MenubarContent>
                <MenubarItem>{t("helpOpen")}</MenubarItem>
                <MenubarSeparator />
                <MenubarItem onClick={openUpdater}>{t("helpUpdater")}</MenubarItem>
                <MenubarItem onClick={openSmapi}>{t("helpSmapi")}</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}