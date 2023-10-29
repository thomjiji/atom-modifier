# Prepare atoms to replace the existing ones

```sh
echo -n -e "\x00\x00\x00\x12colrnclc\x00\x01\x00\x01\x00\x01" > colr_atom_111.bin
echo -n -e "\x00\x00\x00\x12colrnclc\x00\x01\x00\x02\x00\x01" > colr_atom_121.bin
```

```sh
echo -n -e "\x00\x00\x00\x0cgama\x00\x02\x99\x99" > gama_atom_2.6.bin
echo -n -e "\x00\x00\x00\x0cgama\x00\x02\x66\x66" > gama_atom_2.4.bin
echo -n -e "\x00\x00\x00\x0cgama\x00\x02\x33\x33" > gama_atom_2.2.bin
echo -n -e "\x00\x00\x00\x0cgama\x00\x02\x00\x00" > gama_atom_2.0.bin
echo -n -e "\x00\x00\x00\x0cgama\x00\x01\xf5\xc2" > gama_atom_1.96.bin
```

```shell
echo -n -e "\x00\x00\x00\x39\x74\x6D\x63\x64\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x30\x00\x00\x00\x02\x00\x18\x00\x00\x00\x00\x17\x6E\x61\x6D\x65\x00\x0B\x00\x00\x41\x5F\x30\x30\x30\x35\x5f\x31\x32\x53\x4F" > tmcd_atom_addedReelNum.bin
```

# Do the replacement

```sh
./mp4edit \
    --replace \
    "moov/trak/mdia/minf/stbl/stsd/apcn/colr":../atoms/colr_atom_111.bin \
    ../test_footages/no_reel_number/1-1-1_10frames.mov \
    ../test_footages/no_reel_number/1-1-1_10frames_output.mov

./mp4edit \
    --insert \
    "moov/trak/mdia/minf/stbl/stsd/apcn":../atoms/gama_atom_2.2.bin \
    ../test_footages/1-2-1_2frames_apcn_removedGama.mov \
    ../test_footages/1-2-1_2frames_apcn_removedGama_addedAgain2.2.mov

./mp4edit \
    --remove \
    "moov/trak/mdia/minf/stbl/stsd/apcn/gama" \
    ../test_footages/no_reel_number/1-1-1_10frames_output.mov \
    ../test_footages/no_reel_number/1-1-1_10frames_output_removedGama.mov

./mp4edit \
    --replace \
    "moov/trak[1]/mdia/minf/stbl/stsd/tmcd":../atoms/tmcd_atom_addedReelNum.bin \
    ../test_footages/no_reel_number/1-1-1_10frames.mov \
    ../test_footages/no_reel_number/1-1-1_10frames_replacedTmcdAtom.mov
```
