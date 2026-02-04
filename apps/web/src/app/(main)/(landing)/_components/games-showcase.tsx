import Image from "next/image";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

export default function GamesShowcase() {
	return (
		<div className="flex flex-col gap-6 mt-16">
			<div className="self-center flex items-center gap-2 md:gap-3 flex-wrap justify-center">
				<Button className="rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
				<Button className="bg-popover rounded-full">Hot Games</Button>
			</div>
			<div className="relative -mx-4">
				<div className="overflow-x-auto pb-4 px-4 scrollbar-hide">
					<div className="flex gap-4">
						<Game />
						<Game />
						<Game />
						<Game />
						<Game />
						<Game />
						<Game />
					</div>
				</div>
			</div>
			<Image
				src={"/images/footer-seperator.svg"}
				alt="Footer Illustration"
				width={1248}
				height={28}
				className="mb-6 h-4 w-full object-cover sm:mb-8 sm:h-7 lg:mb-12"
			/>
		</div>
	);
}
const Game = () => {
	return (
		<Skeleton className="w-[240px] h-[186px] md:w-[445px] md:h-[334px] rounded-2xl flex-shrink-0" />
	);
};
