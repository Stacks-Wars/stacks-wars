import { PlusCircle } from "lucide-react";
import Image from "next/image";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";

export default function CreateRoom() {
	return (
		<div className="relative rounded-2xl [background:linear-gradient(-28.55deg,_#2c61b8_3.36%,_#000_48.6%,_#000_58.91%,_#000_73.83%,_#2c61b8)] p-3 pt-20! flex flex-col border-darkslategray border-solid border-[0.6px] box-border">
			{/*<div cl70assName="w-full h-[493.7px] relative rounded-[38.8px] [background:linear-gradient(180deg,_#282828,_#000)_border-box] [border:1px_solid_transparent] box-border" />*/}
			<div className="flex items-center flex-col gap-8">
				<Button
					variant={"secondary"}
					size={"lg"}
					className="rounded-full md:py-10 md:max-w-[300px] md:!px-20
					md:text-3xl font-medium md:gap-4 items-center"
				>
					<PlusCircle className="md:!h-8 md:!w-8" />
					Create Room
				</Button>
				<div className="relative">
					<div className="p-4  relative flex items-center border rounded-full bg-muted border-border gap-4">
						<Avatar>
							<AvatarImage src="/icons/shad.svg" />
							<AvatarFallback>PX</AvatarFallback>
						</Avatar>
						<span className="font-normal pr-2">Player_XOXO</span>
					</div>
					<Image
						width={24}
						height={24}
						src={"/icons/location-arrow.svg"}
						className=" absolute -left-2 -top-4"
						alt="Location Arrow"
					/>
				</div>
				<div className="rounded-2xl [background:linear-gradient(126.77deg,_rgba(44,_97,_184,_0),_rgba(0,_0,_0,_0.79)_99.99%,_rgba(0,_0,_0,_0))] border-darkslategray border-solid border-[0.6px] box-border flex flex-col items-center justify-center py-6 px-6 text-xl md:text-2xl">
					<div className="w-full flex flex-col items-start max-w-full shrink-0">
						<b className="self-stretch relative leading-[29.21px] text-center">
							Create or join a game lobby, lock in your STX, and get ready for
							battle.
						</b>
					</div>
				</div>
				{/*<div className="rounded-2xl !p-5 [background:linear-gradient(126.77deg,_rgba(44,_97,_184,_0),_rgba(0,_0,_0,_0.79)_99.99%,_rgba(0,_0,_0,_0))] border-darkslategray border-solid border-[0.6px] box-border flex flex-col items-center justify-center">
					<span className="text-2xl font-bold text-center">
						Create or join a game lobby, lock in your STX, and get ready for
						battle.
					</span>
				</div>*/}
			</div>
		</div>
	);
}
