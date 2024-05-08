import {
    MenubarContent,
    MenubarItem, MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger
} from "@components/ui/menubar.tsx";
import {invoke} from "@tauri-apps/api/tauri";
import {CopyPlus, Globe} from "lucide-react";

export default function File() {

    async function addMod() {
        await invoke("add_mod");
    }

    async function openNexus() {
        await invoke('open_search_browser');
    }

    return (
        <MenubarMenu>
            <MenubarTrigger>File</MenubarTrigger>
            <MenubarContent>
                <MenubarItem onClick={addMod}>
                    <CopyPlus size={14} className="mr-2" />
                    Add Mod
                </MenubarItem>
                <MenubarItem onClick={openNexus}>
                    <Globe size={14} className="mr-2" />
                    Open NexusMod
                </MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Share</MenubarItem>
                <MenubarSeparator />
                <MenubarItem>Print</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}