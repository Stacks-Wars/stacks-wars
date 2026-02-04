import { PlusCircle } from "lucide-react";
import Image from "next/image";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";

export default function CreateRoom() {
	return (
		<div className="px-4 pb-4 pt-9 lg:px-6 lg:pt-14 gap-7 relative border-border-secondary justify-end rounded-3xl lg:rounded-[48px] [background:linear-gradient(-28.55deg,_#2c61b8_3.36%,_#000_48.6%,_#000_58.91%,_#000_73.83%,_#2c61b8)] flex flex-col border-solid border-[0.6px]">
			<Image
				width={354}
				height={491}
				src={"/images/faded-border.png"}
				className=" absolute left-5 right-5 min-[690px]:top-14 object-cover top-4"
				alt="Location Arrow"
			/>
			<Image
				width={354}
				height={491}
				src={"/images/faded-border.png"}
				className=" absolute left-5 right-5 min-[690px]:top-14 object-cover top-4"
				alt="Location Arrow"
			/>
			<Image
				width={354}
				height={491}
				src={"/images/faded-border.png"}
				className="absolute left-7 right-7 object-cover top-6 min-[690px]:top-16"
				alt="Location Arrow"
			/>
			{/*<Image
				width={468}
				height={240}
				src={"/images/faded-border.png"}
				className=" absolute left-5 right-5 object-cover top-4"
				alt="Location Arrow"
			/>*/}
			{/*<div cl70assName="w-full h-[493.7px] relative rounded-[38.8px] [background:linear-gradient(180deg,_#282828,_#000)_border-box] [border:1px_solid_transparent] box-border" />*/}
			<div className="flex items-center flex-col gap-6">
				<Button
					variant={"secondary"}
					className="w-[75%] rounded-full font-medium text-[20px]"
					// className="rounded-full  md:py-10 md:max-w-[300px] md:!px-20
					// md:text-3xl font-medium md:gap-4 items-center"
				>
					<PlusCircle className="" />
					Create Room
				</Button>
				<div className="relative">
					<div className="pl-1.5 py-1.5 pr-3 gap-3 text-xs relative flex items-center border rounded-full bg-muted border-border">
						<Avatar>
							<AvatarImage src="/icons/shad.svg" />
							<AvatarFallback>PX</AvatarFallback>
						</Avatar>
						<span className="font-normal pr-2">Player_XOXO</span>
					</div>
					<Image
						width={12}
						height={12}
						src={"/icons/location-arrow.svg"}
						className="absolute -left-2 -top-1 md:-top-4 md:w-6 md:h-6"
						alt="Location Arrow"
					/>
				</div>
			</div>{" "}
			<div className="rounded-[22px] lg:rounded-[35px] border-border-secondary [background:linear-gradient(126.77deg,_rgba(44,_97,_184,_0),_rgba(0,_0,_0,_0.79)_99.99%,_rgba(0,_0,_0,_0))] border-solid border-[0.6px] box-border flex flex-col items-center justify-center py-6 px-6 md:text-xl">
				<b className="self-stretch relative text-center">
					Create or join a game lobby, lock in your STX, and get ready for
					battle.
				</b>
			</div>
		</div>
	);
}
