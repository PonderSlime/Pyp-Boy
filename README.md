# Pip-Boy
Fully-functional recreation of the pip boy from the Fallout franchise. This has features including:
- Pulse oximeter for heart rate
- GPS for location tracking
- RTL-SDR for hacking the radio waves

I decided to build this project because I had recently came across Fallout 4, which quickly rose to my top 5 video games list. When Highway was annouced, I decided I should build something from it. I had two options: build power armor (which would cost way too much, and is already being done by Hacksmith Industries), or the Pip-Boy (which is much more achievable for several reasons, cost included). Of course I chose the Pip-Boy, because It is still a really cool piece of tech. Another reason to build this was because I thought it would be really cool to navigate an entire system with just knobs, and one button.

**Onshape Link:** [https://cad.onshape.com/documents/fef0e9c1c845178b6cc354c6/w/e4e098e2dc3ae20b4478296f/e/bf3082dc19ffc9c7d7482000?renderMode=0&uiState=6869e2ec55c3557a9e5c5309](https://cad.onshape.com/documents/fef0e9c1c845178b6cc354c6/w/e4e098e2dc3ae20b4478296f/e/bf3082dc19ffc9c7d7482000?renderMode=0&uiState=6869e2ec55c3557a9e5c5309)

## Screenshots
Here's the 3D cad model:

![https://hc-cdn.hel1.your-objectstorage.com/s/v3/2ecbe588889400ed564f8305abd84febf3086739_2025-06-28-181544_hyprshot.png](https://hc-cdn.hel1.your-objectstorage.com/s/v3/2ecbe588889400ed564f8305abd84febf3086739_2025-06-28-181544_hyprshot.png)

Here's the wiring diagram of the components used:

![https://hc-cdn.hel1.your-objectstorage.com/s/v3/c522e84e316c760e37cd8d82a66ab6b510f91d17_2025-07-05-164655_hyprshot.png](https://hc-cdn.hel1.your-objectstorage.com/s/v3/c522e84e316c760e37cd8d82a66ab6b510f91d17_2025-07-05-164655_hyprshot.png)

## BOM:

|Part                                                  |Qty|Link                                                                                                                        |Price  |
|------------------------------------------------------|---|----------------------------------------------------------------------------------------------------------------------------|-------|
|Raspberry Pi Zero 2                                   |1  |https://www.adafruit.com/piz2w                                                                                              |16.50  |
|4.0 Inch Waveshare HDMI Display                       |1  |https://www.amazon.com/HDMI-LCD-Resolution-Resistive-Screen/dp/B07P5H2315                                                   |38.99  |
|V4 RTL-SDR Module                                     |1  |https://www.amazon.com/RTL-SDR-Blog-RTL2832U-Software-Defined/dp/B0CD745394                                                 |37.95  |
|GY-NEO6MV2 NEO-6M GPS Module                          |1  |https://www.amazon.com/DWEII-GY-NEO6MV2-NEO6MV2-Control-Antenna/dp/B0BBM2H5TX/                                              |8.99   |
|MAX10302 Pulse Oximeter                               |1  |https://www.amazon.com/HiLetgo-MAX30102-Breakout-Oximetry-Solution/dp/B07QC67KMQ/                                           |6.99   |
|Lithium Ion Battery - 3.7V 10050mAh                   |1  |https://www.adafruit.com/product/5035                                                                                       |29.95  |
|Power Boost 1000 Charger                              |1  |https://www.adafruit.com/product/2465                                                                                       |19.95  |
|EC12 Encoder                                          |5  |https://www.digikey.com/en/products/detail/alps-alpine/EC12E24204A2/19529077                                                |1.70   |
|Push Button                                           |1  |https://www.digikey.com/en/products/detail/c-k/PVA1-OA-H1-1-2N-V2/417717                                                    |2.40   |
|3mm Diameter 50mm Steel Dowel Pin                     |2  |https://www.mcmaster.com/products/dowel-pins/dowel-pins-1~~/length~50-mm/diameter~3-0000-mm/diameter~3-000-mm/diameter~3-mm/|12.45  |
|3mm Diameter 16mm Steel Dowel Pin                     |1  |https://www.mcmaster.com/products/dowel-pins/dowel-pins-1~~/diameter~3-0000-mm/diameter~3-000-mm/diameter~3-mm/length~16-mm/|10.84  |
|3mm Diameter 7mm Steel Dowel Pin                      |2  |https://www.mcmaster.com/products/dowel-pins/dowel-pins-1~~/diameter~3-0000-mm/diameter~3-000-mm/diameter~3-mm/length~7-mm/ |6.83   |
|3mm Diameter 36mm Steel Dowel Pin                     |1  |https://www.mcmaster.com/products/dowel-pins/dowel-pins-1~~/diameter~3-0000-mm/diameter~3-000-mm/diameter~3-mm/length~36-mm/|11.96  |
|3mm Diameter 30mm Steel Dowel Pin                     |1  |https://www.mcmaster.com/products/dowel-pins/dowel-pins-1~~/diameter~3-0000-mm/diameter~3-000-mm/diameter~3-mm/length~30-mm/|16.40  |
|M3x4mmx5mm Heatset Inserts                            |6  |https://www.amazon.com/Threaded-Inserts-Printing-Components-Assortment/dp/B0DGQH7YX6                                        |7.09   |
|M3 Screw Kit (whatever screw length works)            |1  |https://www.amazon.com/Assortment-Stainless-Replacement-Machine-Fastener/dp/B0CMQG542V                                      |15.99  |
|2 Gallon Polypropylene Container (for electroplating) |1  |https://www.mcmaster.com/products/tubs/food-industry-plastic-storage-containers-6/material~polypropylene/                   |9.19   |
|Polypropylene Carboy (Copper Sulfate solution storage)|1  |https://www.mcmaster.com/products/tanks/capacity~3-gal/carboys-1~/                                                          |31.29  |
|Copper sulfate Pentahydrate                           |1  |https://www.amazon.com/Copper-Sulfate-Pentahydrate-Powder-Lbs/dp/B018W893PY/                                                |22.95  |
|Distilled Water                                       |2  |https://www.walmart.com/ip/Pure-Life-Distilled-Water-1-Gallon-Plastic-Bottled-Water-1-Pack-Side-Handle/1070666864           |2.38   |
|Distilled White Vinegar                               |2  |https://www.walmart.com/ip/Great-Value-Distilled-White-Vinegar-128-fl-oz/10450998                                           |3.94   |
|Metal Polish                                          |1  |https://www.amazon.com/Brasso-2660089334-Multi-Purpose-Metal-Polish/dp/B00D600PLA/                                          |4.98   |
|TOTAL                                                 |   |                                                                                                                            |$309.71|

