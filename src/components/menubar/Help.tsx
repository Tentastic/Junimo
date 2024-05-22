import {
    MenubarContent,
    MenubarItem, MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger
} from "@components/ui/menubar.tsx";
import {useTranslation} from "react-i18next";


export default function Help() {
    const { t } = useTranslation('home');

    return (
        <MenubarMenu>
            <MenubarTrigger>{t("help")}</MenubarTrigger>
            <MenubarContent>
                <MenubarItem>Open Help</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Check for Update</MenubarItem>
                <MenubarItem>Check for Smapi Update</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}