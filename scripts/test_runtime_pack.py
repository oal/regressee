#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
import importlib.util, os, tempfile, unittest, subprocess, json, zipfile
from pathlib import Path
spec=importlib.util.spec_from_file_location('runtime_pack',Path(__file__).with_name('runtime-pack.py'));pack=importlib.util.module_from_spec(spec);spec.loader.exec_module(pack)
class PackTests(unittest.TestCase):
    def test_deterministic_private_candidate_owner_signature_and_atomic_no_clobber(self):
        with tempfile.TemporaryDirectory(prefix='phase7-pack-') as d:
            parent=Path(d);parent.chmod(0o700);root=parent/'root';root.mkdir(mode=0o700)
            (root/'node').write_bytes(b'inert node');(root/'worker.mjs').write_bytes(b'inert worker')
            browser=root/'browsers/A.app/F.framework/Versions/A';browser.mkdir(parents=True)
            (browser/'F').write_bytes(b'inert framework');(browser/'F').chmod(0o755)
            (browser.parent/'Current').symlink_to('A')
            base={'id':'inert-phase7','os':'linux','arch':'x86_64','scope':'DevelopmentLoopback','support_until':None,
                'identity':{'Pinned':{'node':'1.0.0','worker':'1.0.0','playwright':'1.0.0','browser':'1.0.0'}},'node':'node','worker':'worker.mjs','browser_root':'browsers'}
            a=parent/'a';b=parent/'b';pack.prepare(root,base,a);pack.prepare(root,base,b)
            self.assertEqual((a/'manifest.json').read_bytes(),(b/'manifest.json').read_bytes());self.assertEqual((a/'payload.zip').read_bytes(),(b/'payload.zip').read_bytes())
            def openssl(*args):subprocess.run(['/usr/bin/openssl',*map(str,args)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=15)
            # Ephemeral fixture keys live only in this scoped disposable test tree.
            key=parent/'test.key';pub=parent/'public.pem';sig=parent/'signature.bin'
            openssl('genpkey','-algorithm','ED25519','-out',key);openssl('pkey','-in',key,'-pubout','-out',pub)
            openssl('pkeyutl','-sign','-inkey',key,'-rawin','-in',a/'manifest.json','-out',sig)
            output=parent/'runtime.zip';digest=pack.assemble(a,sig,pub,output);original=output.read_bytes()
            with zipfile.ZipFile(output) as z:
                self.assertIn('manifest.sig',z.namelist());self.assertIn('manifest.json',z.namelist())
            with self.assertRaises(FileExistsError):pack.assemble(a,sig,pub,output)
            self.assertEqual(output.read_bytes(),original)
            sig.write_bytes(bytes(64))
            with self.assertRaises(pack.Refusal):pack.assemble(a,sig,pub,parent/'invalid.zip')
            self.assertFalse((parent/'invalid.zip').exists())
            (root/'bad').symlink_to('/etc/passwd')
            with self.assertRaises(pack.Refusal):pack.prepare(root,base,parent/'hostile')
            self.assertFalse((parent/'hostile').exists())
    def test_contained_framework_and_malicious_links(self):
        with tempfile.TemporaryDirectory(prefix='phase7-pack-') as d:
            root=Path(d)/'root';p=root/'browsers/A.app/F.framework/Versions/A';p.mkdir(parents=True)
            (p/'F').write_bytes(b'inert');(p/'F').chmod(0o755)
            link=p.parent/'Current';link.symlink_to('A')
            files,links,modes=pack.inventory(root)
            self.assertEqual(links['browsers/A.app/F.framework/Versions/Current'],'A');self.assertEqual(modes['browsers/A.app/F.framework/Versions/A/F'],0o755)
            for target in ['/etc/passwd','../../../../../outside','Current','missing']:
                link.unlink();link.symlink_to(target)
                with self.assertRaises(pack.Refusal):pack.inventory(root)
    def test_case_mode_leaf_and_special_refuse(self):
        with tempfile.TemporaryDirectory(prefix='phase7-pack-') as d:
            root=Path(d);(root/'a').write_bytes(b'x');(root/'A').write_bytes(b'x')
            with self.assertRaises(pack.Refusal):pack.inventory(root)
            (root/'A').unlink();(root/'a').chmod(0o4777)
            with self.assertRaises(pack.Refusal):pack.inventory(root)
            (root/'a').unlink();os.mkfifo(root/'fifo')
            with self.assertRaises(pack.Refusal):pack.inventory(root)
if __name__=='__main__':unittest.main()
