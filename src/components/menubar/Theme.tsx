import {MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarTrigger} from "@components/ui/menubar.tsx";
import {useTranslation} from "react-i18next";


export default function Theme() {
    const { t } = useTranslation('home');

    function setTheme(color: string) {
        const html = document.documentElement;
        const current = localStorage.getItem('theme');
        if (current !== null) html.classList.remove(current);
        html.classList.add(color);
        localStorage.setItem('theme', color);
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>{t("theme")}</MenubarTrigger>
            <MenubarContent>
                <MenubarItem onClick={x => setTheme("light")}>{t("lightTheme")}</MenubarItem>
                <MenubarItem onClick={x => setTheme("dark")}>{t("darkTheme")}</MenubarItem>
                <MenubarItem onClick={x => setTheme("black")}>{t("blackTheme")}</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}