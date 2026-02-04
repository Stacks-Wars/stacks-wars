import Image from "next/image";

export default function WinStx() {
	return (
		<div className="w-full rounded-3xl lg:rounded-[48px] min-[690px]:col-span-2 h-[326.5px] md:h-fit md:relative md:flex md:items-center flex-row-reverse relative shadow-[0px_0px_34.07px_-18.09px_rgba(255,_255,_255,_0.25)_inset] [background:linear-gradient(150.88deg,_#2c61b8,_#000_65.11%)] md:[background:linear-gradient(98.77deg,_#2c61b8,_#000_36.25%,_#1d2e4e_62.38%,_#2c61b8_96.98%)] border-border-secondary border-solid border overflow-hidden">
			<div className="md:hidden relative h-full">
				<div className="rounded-[7.82px] flex items-center justify-center m-4 border-border-secondary border-solid border py-4 px-4">
					<b className="text-center text-[19.6px]">
						<span>
							Win STX and rise in the ranks. The top warriors take home the
							prize.
						</span>
					</b>
				</div>
				<Image
					className="absolute bottom-0 left-4 right-4 md:-left-10 object-cover shrink-0"
					src={"/images/trophy.png"}
					width={416}
					height={624}
					alt=""
				/>
			</div>
			<div className="relative w-full h-full hidden md:flex py-14 lg:py-20 md:pl-14 pr-8 flex-row-reverse items-center">
				<div className="rounded-2xl pl-60 text-center pr-4 border-border-secondary border-solid border py-8">
					<b className="text-[19.6px] lg:text-2xl relative text-wrap">
						<span>
							Win STX and rise in the ranks. The top warriors take home the
							prize.
						</span>
					</b>
				</div>
				<Image
					className="absolute bottom-0 md:-left-20 object-cover z-[5]"
					src="/images/trophy.png"
					width={416}
					height={624}
					alt="Trophy"
					quality={95}
					sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
					unoptimized
				/>
			</div>
		</div>
	);
}
