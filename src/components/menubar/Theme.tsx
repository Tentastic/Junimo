import {MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarTrigger} from "@components/ui/menubar.tsx";


export default function Theme() {
    function setTheme(color: string) {
        const html = document.documentElement;
        const current = localStorage.getItem('theme');
        if (current !== null) html.classList.remove(current);
        html.classList.add(color);
        localStorage.setItem('theme', color);
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>Theme</MenubarTrigger>
            <MenubarContent>
                <MenubarItem onClick={x => setTheme("light")}>Light</MenubarItem>
                <MenubarItem onClick={x => setTheme("dark")}>Dark</MenubarItem>
                <MenubarItem onClick={x => setTheme("black")}>Black</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}