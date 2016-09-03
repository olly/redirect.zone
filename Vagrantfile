# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  config.vm.box = "centos/7"

  # config.vm.synced_folder ".", "/home/vagrant/redirect.zone", type: "rsync"

  config.vm.provision "shell", inline: <<-SHELL
    set -e

    sudo yum install --assumeyes clang gcc systemd-devel wget
    if [ ! -f rust-1.11.0-x86_64-unknown-linux-gnu.tar.gz ]; then
      wget --continue -q https://static.rust-lang.org/dist/rust-1.11.0-x86_64-unknown-linux-gnu.tar.gz
      tar xzf rust-1.11.0-x86_64-unknown-linux-gnu.tar.gz
      sudo ./rust-1.11.0-x86_64-unknown-linux-gnu/install.sh
    fi;
  SHELL
end
