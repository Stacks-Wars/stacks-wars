import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function TryOut() {
	return (
		<div className="py-28 relative px-8 md:px-0 flex items-center justify-center">
			<Image
				src={"/images/rings.png"}
				width={860}
				height={860}
				alt="Colorful rings background behind the featured game call to action"
				className="object-cover"
			/>
			<div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-center flex-col gap-3">
				<span className="font-bold text-2xl md:text-3xl lg:text-4xl text-center">
					Try Out Our First Featured Game!
				</span>
				<Button variant={"outline"} asChild className="rounded-full">
					<Link href={"/games"}>Play Now!</Link>
				</Button>
			</div>
		</div>
	);
}
