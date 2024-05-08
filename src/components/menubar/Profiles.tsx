import {
    MenubarContent,
    MenubarItem,
    MenubarMenu,
    MenubarRadioGroup, MenubarRadioItem,
    MenubarSeparator,
    MenubarTrigger
} from "@components/ui/menubar.tsx";
import {invoke} from "@tauri-apps/api/tauri";
import {Profile} from "@models/profile.ts";
import {Dispatch, SetStateAction, useEffect, useState} from "react";
import {Download} from "@models/download.ts";
import {listen} from "@tauri-apps/api/event";


export default function Profiles({setKey}: {setKey: Dispatch<SetStateAction<number>>}) {
    const [profiles, setProfiles] = useState<Profile[]>([]);
    const [selectedProfile, setSelectedProfile] = useState<string>("Default");

    async function loadProfile() {
        const loadedProfiles = await invoke<Profile[]>('get_profiles');
        for (let profile of profiles) {
            if (profile.currently) {
                setSelectedProfile(profile.name);
                break;
            }
        }
        setProfiles(loadedProfiles);
    }

    async function openProfiles() {
        await invoke("open_profile")
    }

    async function changeProfile(name: string) {
        const newLoadedProfiles = await invoke<Profile[]>('change_current_profile', {name: name});
        setProfiles(newLoadedProfiles);
        setSelectedProfile(name);
        setKey(prevKey => prevKey + 1);
    }

    useEffect(() => {
        loadProfile();

        const handleNewData = (event: any) => {
            const data = event.payload as Profile[];
            for (let profile of data) {
                if (profile.currently) {
                    setSelectedProfile(profile.name);
                    break;
                }
            }
            setProfiles(data);
            setKey(prevKey => prevKey + 1);
        };

        let unsubscribeEvent = listen('profile-update', handleNewData);

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
        };
    }, []);

    return (
        <MenubarMenu>
            <MenubarTrigger>Profile</MenubarTrigger>
            <MenubarContent>
                <MenubarRadioGroup value={selectedProfile}>
                    {
                        profiles.map((profile, index) => (
                            <MenubarRadioItem key={index} value={profile.name} onClick={i => changeProfile(profile.name)}>
                                {profile.name}
                            </MenubarRadioItem>
                        ))
                    }
                </MenubarRadioGroup>
                <MenubarSeparator />
                <MenubarItem onClick={openProfiles}>Edit Profiles</MenubarItem>
            </MenubarContent>
        </MenubarMenu>
    )
}