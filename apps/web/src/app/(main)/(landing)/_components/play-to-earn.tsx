import CreateRoom from "./create-room";
import Gameplay from "./gameplay";
import WinStx from "./win-stx";

export default function PlayToEarn() {
	return (
		<div className="grid grid-cols-1 min-[690px]:grid-cols-2 gap-6 mt-16 mx-auto p-4 w-full">
			<CreateRoom />
			<Gameplay />
			<WinStx />
		</div>
	);
}
