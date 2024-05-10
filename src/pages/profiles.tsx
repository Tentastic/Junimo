import {invoke} from "@tauri-apps/api/core";
import {Profile} from "@models/profile.ts";
import {useEffect, useState} from "react";
import { Trash2, Pencil } from "lucide-react"
import {
    Dialog, DialogClose,
    DialogContent,
    DialogDescription, DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@components/ui/dialog"
import {Input} from "@components/ui/input.tsx";


export default function Profiles() {
    const [profiles, setProfiles] = useState<Profile[]>([]);
    const [newProfile, setNewProfile] = useState<string>("");

    async function loadProfile() {
        const loadedProfiles = await invoke<Profile[]>('get_profiles');
        setProfiles(loadedProfiles);
    }

    async function addProfile() {
        const newLoadedProfiles = await invoke<Profile[]>('add_profile', {name: newProfile});
        setProfiles(newLoadedProfiles);
        setNewProfile("");
    }

    async function removeProfile(name: string) {
        const newLoadedProfiles = await invoke<Profile[]>('remove_profile', {name: name});
        setProfiles(newLoadedProfiles);
    }

    async function modifyProfile(name: string) {
        const newLoadedProfiles = await invoke<Profile[]>('modify_profile', {name: name, newName: newProfile});
        setProfiles(newLoadedProfiles);
        setNewProfile("");
    }

    async function changeProfile(name: string) {
        const newLoadedProfiles = await invoke<Profile[]>('change_current_profile', {name: name});
        setProfiles(newLoadedProfiles);

    }

    useEffect(() => {
        loadProfile();
    }, []);

    return (
        <div className="w-full h-[100vh] p-6 flex flex-col justify-start items-start">
            <h1 className="text-3xl text-left font-bold mb-4">Profiles</h1>
            <div className="w-full flex flex-col gap-2">
                {profiles.map((profile, index) => (
                    <div key={index} className="flex items-center gap-2">
                        <div className="h-5 w-5 text-sm cursor-pointer" onClick={i => changeProfile(profile.name)}>
                            {profile.currently ? "🟢" : "⚫"}
                        </div>
                        <p>{profile.name}</p>
                        <p>(Mods active: {profile.mods.length})</p>
                        <div className="ml-auto">
                            <Dialog>
                                <DialogTrigger className="h-4" onClick={i => setNewProfile("")}>
                                    <button className="ml-auto brightness-95 hover:brightness-75 p-2 rounded bg-muted">
                                        <Pencil size={18}/>
                                    </button>
                                </DialogTrigger>
                                <DialogContent>
                                    <DialogHeader>
                                        <DialogTitle className="mb-4">Edit Name from Profile {profile.name}</DialogTitle>
                                        <DialogDescription>
                                            <Input id="path" placeholder="Please enter the new name of the profile..."
                                                   value={newProfile} onChange={x => setNewProfile(x.target.value)} />
                                        </DialogDescription>
                                    </DialogHeader>
                                    <DialogFooter className="sm:justify-end pt-2">
                                        <DialogClose asChild>
                                            <button onClick={i => modifyProfile(profile.name)}
                                                    className="transition duration-300 brightness-95 text-foreground bg-primary hover:brightness-75 p-2 px-4 rounded">Save
                                            </button>
                                        </DialogClose>
                                    </DialogFooter>
                                </DialogContent>
                            </Dialog>
                            {
                                !profile.currently && profile.name !== "Default" ?
                                    (
                                        <Dialog>
                                            <DialogTrigger className="h-4 ml-2">
                                                <button className="ml-auto brightness-95 hover:brightness-75 p-2 rounded bg-muted">
                                                    <Trash2 size={18}/>
                                                </button>
                                            </DialogTrigger>
                                            <DialogContent>
                                                <DialogHeader>
                                                    <DialogTitle className="mb-4">Are you absolutely sure?</DialogTitle>
                                                    <DialogDescription>
                                                        This action cannot be undone. This will permanently delete this profile and all selected mods.
                                                    </DialogDescription>
                                                </DialogHeader>
                                                <DialogFooter className="sm:justify-end">
                                                    <DialogClose asChild>
                                                        <button onClick={i => removeProfile(profile.name)}
                                                                className="transition duration-300 text-foreground bg-destructive hover:brightness-75 p-2 px-4 rounded">
                                                            Delete
                                                        </button>
                                                    </DialogClose>
                                                </DialogFooter>
                                            </DialogContent>
                                        </Dialog>
                                    ) : ""
                            }
                        </div>
                    </div>
                ))}
                <div className="w-full h-[2px] bg-border rounded mt-4" />
                <Dialog>
                    <DialogTrigger className="w-full" onClick={i => setNewProfile("")}>
                        <button className="w-full bg-muted rounded p-2 mt-4">Add Profile</button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle className="mb-4">New Profile</DialogTitle>
                            <DialogDescription>
                                <Input id="path" placeholder="Please enter the name of the profile..."
                                       value={newProfile} onChange={x => setNewProfile(x.target.value)} />
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter className="sm:justify-end pt-2">
                            <DialogClose asChild>
                                <button onClick={addProfile}
                                        className="transition duration-300 brightness-95 text-foreground bg-primary hover:brightness-75 p-2 px-4 rounded">Save
                                </button>
                            </DialogClose>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>
        </div>
    )
}