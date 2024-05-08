import {MenubarContent, MenubarItem, MenubarMenu, MenubarSeparator, MenubarTrigger} from "@components/ui/menubar.tsx";
import {Settings} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";


export default function Tools() {
    async function openConfig() {
        await invoke("open_config")
    }

    async function openExport() {
        await invoke("open_export")
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>Tools</MenubarTrigger>
            <MenubarContent>
                <MenubarItem>Import</MenubarItem>
                <MenubarItem onClick={openExport}>Export</MenubarItem>
                <MenubarSeparator />
                <MenubarItem onClick={openConfig}>
                    <Settings size={16} className="mr-1" /> Settings
                </MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}