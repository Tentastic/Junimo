import {useRef, useState} from "react";
import Logo from "../assets/Logo.webp";
import { invoke } from "@tauri-apps/api/tauri";
import "../App.css";
import { Settings, ArrowLeft, ArrowRight, Play, Pause } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@components/ui/tabs"

import {
    Menubar,
    MenubarContent,
    MenubarItem,
    MenubarMenu,
    MenubarSeparator,
    MenubarShortcut,
    MenubarTrigger,
} from "@components/ui/menubar"
import Downloader from "@components/Downloader.tsx";
import ModsInstalled from "@components/ModsInstalled.tsx";
import {clsx} from "clsx";
import Mods from "@components/Mods.tsx";
import {Profile} from "@models/profile.ts";
import {ModInfos} from "@models/mods.ts";
import Console from "@components/Console.tsx";
import {Download} from "@models/download.ts";
import Theme from "@components/Theme.tsx";

function Home() {
    const [key, setKey] = useState(0);
    const [name, setName] = useState("");
    const [selectedInstalled, setSelectedInstalled] = useState<number[]>([]);
    const [selectedMods, setSelectedMods] = useState<number[]>([]);
    const [profile, setProfile] = useState<Profile>();
    const [modList, setModList] = useState<ModInfos[]>([]);
    const [downloadList, setDownloadList] = useState<Download[]>([]);
    const [playing, setPlaying] = useState(false);

    window.addEventListener('contextmenu', function (e) {
        e.preventDefault();  // This will prevent the default context menu
    }, false);

    async function test() {
        const now = new Date(); // Get the current date and time

        // Get the current minute and second
        let minutes = now.getMinutes();
        let seconds = now.getSeconds();

        // Combine into a single string
        const timeString = `${minutes}:${seconds}`;

        await invoke("test", {name: timeString});
    }

  async function openConfig() {
      await invoke("open_config")
  }

  async function remove() {
    if (profile !== undefined) {
        const mods = profile.mods.filter((x, i) => !selectedMods.includes(i));
        await invoke('change_profile_mods', {name: profile.name, mods: mods});
        setSelectedMods([]);
        setKey(prevKey => prevKey + 1);
    }
  }

async function add() {
    if (profile !== undefined) {
        const mods = profile.mods.concat(modList.filter((x, i) => selectedInstalled.includes(i)));
        await invoke('change_profile_mods', {name: profile.name, mods: mods});
        setSelectedInstalled([]);
        setKey(prevKey => prevKey + 1);
    }
  }

  async function start() {
      setPlaying(true);
      await invoke('start_game');
  }

  async function stop() {
      await invoke('stop_game');
      setPlaying(false);
  }

  return (
    <div className="w-full h-[100vh] flex flex-col">
        <Menubar>
            <MenubarMenu>
                <MenubarTrigger>File</MenubarTrigger>
                <MenubarContent>
                    <MenubarItem>
                        New Tab <MenubarShortcut>⌘T</MenubarShortcut>
                    </MenubarItem>
                    <MenubarItem>New Window</MenubarItem>
                    <MenubarSeparator />
                    <MenubarItem>Share</MenubarItem>
                    <MenubarSeparator />
                    <MenubarItem>Print</MenubarItem>
                </MenubarContent>
            </MenubarMenu>
            <MenubarMenu>
                <MenubarTrigger>Tools</MenubarTrigger>
                <MenubarContent>
                    <MenubarItem onClick={openConfig}>
                        <Settings size={16} className="mr-1" /> Settings
                    </MenubarItem>
                </MenubarContent>
            </MenubarMenu>
            <MenubarMenu>
                <MenubarTrigger>Profile</MenubarTrigger>
                <MenubarContent>
                    <MenubarItem>
                        Default
                    </MenubarItem>
                    <MenubarItem>New Window</MenubarItem>
                    <MenubarItem>Share</MenubarItem>
                    <MenubarSeparator />
                    <MenubarItem>Modfiy</MenubarItem>
                </MenubarContent>
            </MenubarMenu>
            <Theme />
            <MenubarMenu>
                <MenubarTrigger>Help</MenubarTrigger>
                <MenubarContent>
                    <MenubarItem>
                        New Tab <MenubarShortcut>⌘T</MenubarShortcut>
                    </MenubarItem>
                    <MenubarItem>New Window</MenubarItem>
                    <MenubarSeparator />
                    <MenubarItem>Share</MenubarItem>
                    <MenubarSeparator />
                    <MenubarItem>Print</MenubarItem>
                </MenubarContent>
            </MenubarMenu>
        </Menubar>
        <main className={clsx(
            "flex-grow flex-1 p-6 pt-2 grid gap-4 grid-cols-[1fr_1fr_auto_1fr]",
            playing ? "grid-rows-[auto_auto_2fr]" : "grid-rows-[auto_1fr_auto]"
        )}>
            <div className="flex col-span-3 min-h-12 items-center justify-between">
                <div className="flex items-center gap-2 mb-2">
                    <img src={Logo} alt="React Logo" className="w-10 h-10 select-none"/>
                    <h1 className="text-4xl mt-2 text-[#8bc24a] font-bold select-none">Junimo</h1>
                </div>
                <div className="mr-16">
                    { playing ? (
                            <button onClick={stop} className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                <Pause size={24} />
                            </button>
                    ):
                        (
                            <button onClick={start}
                                    className="p-2 rounded-lg transition duration-150 bg-muted hover:bg-muted-dark">
                                <Play size={24}/>
                            </button>
                        )
                    }

                </div>
            </div>
            <Tabs defaultValue="mods" className="row-span-2 flex flex-col mt-2">
                <TabsList>
                <TabsTrigger value="mods">Mods</TabsTrigger>
                    <TabsTrigger value="download">Downloads</TabsTrigger>
                </TabsList>
                <TabsContent value="mods" className="flex-1 mt-7">
                    <div className="relative flex-grow h-full p-4 pr-0 px-3 pt-0 border-border border rounded-lg">
                        <div className="absolute -top-5 left-2 bg-background p-2 px-4">
                            <h2 className="text-lg">Mods Installed</h2>
                        </div>
                        <ModsInstalled modList={modList} setModList={setModList} selected={selectedInstalled} setSelected={setSelectedInstalled} key={key} className={playing ? "max-h-[30vh]" : "max-h-[50vh]"} />
                    </div>
                </TabsContent>
                <TabsContent value="download" className="flex-1 mt-7">
                    <div className="relative flex-grow h-full p-4 pr-0 pt-0 border-border border rounded-lg">
                        <div className="absolute -top-5 left-2 bg-background p-2 px-4">
                            <h2 className="text-lg">Downloads</h2>
                        </div>
                        <Downloader downloadList={downloadList} setDownloadList={setDownloadList} className={playing ? "max-h-[30vh]" : ""} />
                    </div>
                </TabsContent>
            </Tabs>
            <div className="relative w-full h-full border-border p-4 pr-0 pt-0 border rounded-lg col-span-2">
                <div className="absolute -top-5 bg-background left-2 p-2 px-4">
                    <h2 className="text-lg">Current Mods</h2>
                </div>
                <Mods profile={profile} setProfile={setProfile} selected={selectedMods} setSelected={setSelectedMods} key={key} className={playing ? "max-h-[30vh]" : "max-h-[67vh]"} />
            </div>
            <div className="flex flex-col gap-2 h-full items-center justify-center">
                <button onClick={add} className={clsx(
                    "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                    selectedInstalled.length === 0 && "opacity-50 pointer-events-none"
                )}>
                    <ArrowLeft />
                </button>
                <button onClick={remove} className={clsx(
                    "h-12 w-12 rounded-full transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center",
                    selectedMods.length === 0 && "opacity-50 pointer-events-none"
                )}>
                    <ArrowRight />
                </button>
            </div>

            <div className={clsx(
                "h-32 border-border transition duration-150 cursor-pointer hover:bg-muted pl-2 col-span-4 border rounded-lg",
                playing ? "h-[50vh]" : "h-32"
            )}>
                <Console />
            </div>
        </main>
    </div>
  );
}

export default Home;
